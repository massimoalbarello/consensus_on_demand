
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use std::thread::{Builder as ThreadBuilder, JoinHandle};

use crate::consensus_layer::{ConsensusProcessor, artifacts::{ConsensusMessage, UnvalidatedArtifact}};

// Periodic duration of `PollEvent` in milliseconds.
const ARTIFACT_MANAGER_TIMER_DURATION_MSEC: u64 = 200;

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

// Manages the life cycle of the client specific artifact processor thread
pub struct ArtifactProcessorManager {
    // The list of unvalidated artifacts
    pending_artifacts: Arc<Mutex<Vec<UnvalidatedArtifact<ConsensusMessage>>>>,
    // To send the process requests
    sender: Sender<ProcessRequest>,
    // Handle for the processing thread
    handle: Option<JoinHandle<()>>,
}

impl ArtifactProcessorManager {
    pub fn new(node_number: u8) -> Self {

        let pending_artifacts = Arc::new(Mutex::new(Vec::new()));
        let (sender, receiver) = crossbeam_channel::unbounded::<ProcessRequest>();

        let client = Box::new(ConsensusProcessor::new(node_number));

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
        pending_artifacts: Arc<Mutex<Vec<UnvalidatedArtifact<ConsensusMessage>>>>,
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

                    let (adverts, result) = client.process_changes(artifacts);

                    if let ProcessingResult::StateChanged = result {
                        sender
                            .send(ProcessRequest)
                            .unwrap_or_else(|err| panic!("Failed to send request: {:?}", err));
                    }
                    adverts.into_iter().for_each(|adv| {
                        println!("Message to be broadcasted: {:?}", adv);
                        // use a channel to send messages to network layer so that it can broadcast them
                    });
                },
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    pub fn on_artifact(&self, artifact: UnvalidatedArtifact<ConsensusMessage>) {
        println!("Received artifact added to pending artifacts");
        let mut pending_artifacts = self.pending_artifacts.lock().unwrap();
        pending_artifacts.push(artifact);
        self.sender.send(ProcessRequest).unwrap_or_else(|err| panic!("Failed to send request: {:?}", err));;
    }
}