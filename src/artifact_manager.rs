
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use std::thread::{Builder as ThreadBuilder, JoinHandle};

use crate::{consensus_layer::{
    ConsensusProcessor,
    artifacts::{ConsensusMessage, UnvalidatedArtifact}
}, time_source::{TimeSource, SysTimeSource}};

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
    sender_incoming_request: Sender<ProcessRequest>,
    // Handle for the processing thread
    handle: Option<JoinHandle<()>>,
}

impl ArtifactProcessorManager {
    pub fn new(node_number: u8, time_source: Arc<SysTimeSource>, sender_outgoing_artifact: Sender<ConsensusMessage>) -> Self {

        let pending_artifacts = Arc::new(Mutex::new(Vec::new()));
        let (sender_incoming_request, receiver_incoming_request) = crossbeam_channel::unbounded::<ProcessRequest>();

        let client = Box::new(ConsensusProcessor::new(node_number, Arc::clone(&time_source) as Arc<_>));

        // Spawn the processor thread
        let sender_incoming_request_cl = sender_incoming_request.clone();
        let pending_artifacts_cl = pending_artifacts.clone();
        let handle = ThreadBuilder::new()
            .spawn(move || {
                Self::process_messages(
                    pending_artifacts_cl,
                    time_source,
                    client,
                    sender_incoming_request_cl,
                    receiver_incoming_request,
                    sender_outgoing_artifact,
                );
            })
            .unwrap();

        Self {
            pending_artifacts,
            sender_incoming_request,
            handle: Some(handle),
        }
    }

    fn process_messages(
        pending_artifacts: Arc<Mutex<Vec<UnvalidatedArtifact<ConsensusMessage>>>>,
        time_source: Arc<SysTimeSource>,
        client: Box<ConsensusProcessor>,
        sender_incoming_request: Sender<ProcessRequest>,
        receiver_incoming_request: Receiver<ProcessRequest>,
        sender_outgoing_artifact: Sender<ConsensusMessage>,
    ) {
        println!("Incoming artifacts thread loop started");
        let recv_timeout = std::time::Duration::from_millis(ARTIFACT_MANAGER_TIMER_DURATION_MSEC);
        loop {
            let ret = receiver_incoming_request.recv_timeout(recv_timeout);

            match ret {
                Ok(_) | Err(RecvTimeoutError::Timeout) => {
                    let artifacts = {
                        let mut artifacts = Vec::new();
                        let mut received_artifacts = pending_artifacts.lock().unwrap();
                        std::mem::swap(&mut artifacts, &mut received_artifacts);
                        artifacts
                    };

                    let (adverts, result) = client.process_changes(time_source.as_ref(), artifacts);

                    if let ProcessingResult::StateChanged = result {
                        sender_incoming_request
                            .send(ProcessRequest)
                            .unwrap_or_else(|err| panic!("Failed to send request: {:?}", err));
                    }
                    adverts.into_iter().for_each(|adv| {
                        // use channel to send locally generated artifacts to network layer so that it can broadcast them
                        sender_outgoing_artifact.send(adv).unwrap_or_else(|err| panic!("Failed to send artifact: {:?}", err));
                    });
                },
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    pub fn on_artifact(&self, artifact: UnvalidatedArtifact<ConsensusMessage>) {
        let mut pending_artifacts = self.pending_artifacts.lock().unwrap();
        pending_artifacts.push(artifact);
        self.sender_incoming_request.send(ProcessRequest).unwrap_or_else(|err| panic!("Failed to send request: {:?}", err));
    }
}