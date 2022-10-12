pub mod processor {
    use std::sync::{Arc, Mutex};
    use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
    use std::thread::{Builder as ThreadBuilder, JoinHandle};

    use crate::consensus_layer::blockchain::{ConsensusProcessor, Artifact, UnvalidatedArtifact};

    // Periodic duration of `PollEvent` in milliseconds.
    const ARTIFACT_MANAGER_TIMER_DURATION_MSEC: u64 = 1000;

    struct ProcessRequest;

    // The result of a single 'process_changes' call can result in either:
    // - new changes applied to the state. So 'process_changes' should be
    //   immediately called again.
    // - no change applied and state was unchanged. So calling 'process_changes' is
    //   not immediately required.
    pub enum ProcessingResult {
        StateChanged,
        StateUnchanged,
    }

    pub struct ArtifactProcessorManager {
        pending_artifacts: Arc<Mutex<Vec<UnvalidatedArtifact<Artifact>>>>,
        sender: Sender<ProcessRequest>,
        handle: Option<JoinHandle<()>>,
    }

    impl ArtifactProcessorManager {
        pub fn new() -> Self {

            let pending_artifacts = Arc::new(Mutex::new(Vec::new()));
            let (sender, receiver) = crossbeam_channel::unbounded::<ProcessRequest>();

            let client = Box::new(ConsensusProcessor::new());

            // Spawn the processor thread
            let sender_cl = sender.clone();
            let pending_artifacts_cl = pending_artifacts.clone();
            let handle = ThreadBuilder::new()
                .spawn(move || {
                    Self::process_messages(
                        pending_artifacts_cl,
                        client,
                        sender_cl,
                        receiver,
                    );
                })
                .unwrap();

            Self {
                pending_artifacts,
                sender,
                handle: Some(handle),
            }
        }

        fn process_messages(
            pending_artifacts: Arc<Mutex<Vec<UnvalidatedArtifact<Artifact>>>>,
            client: Box<ConsensusProcessor>,
            sender: Sender<ProcessRequest>,
            receiver: Receiver<ProcessRequest>,
        ) {
            println!("Thread loop started");
            let recv_timeout = std::time::Duration::from_millis(ARTIFACT_MANAGER_TIMER_DURATION_MSEC);
            loop {
                let ret = receiver.recv_timeout(recv_timeout);
    
                match ret {
                    Ok(_) | Err(RecvTimeoutError::Timeout) => {
                        let artifacts = {
                            let mut artifacts = Vec::new();
                            let mut received_artifacts = pending_artifacts.lock().unwrap();
                            std::mem::swap(&mut artifacts, &mut received_artifacts);
                            artifacts
                        };
    
                        let result = client.process_changes(artifacts);
    
                        if let ProcessingResult::StateChanged = result {
                            sender
                                .send(ProcessRequest)
                                .unwrap_or_else(|err| panic!("Failed to send request: {:?}", err));
                        }
                    },
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }
        }

        pub fn on_artifact(&self, artifact: UnvalidatedArtifact<Artifact>) {
            println!("Received artifact added to pending artifacts");
            let mut pending_artifacts = self.pending_artifacts.lock().unwrap();
            pending_artifacts.push(artifact);
            self.sender.send(ProcessRequest);
        }
    }
}