use tokio::time::{sleep, Duration};

const AGENT_SLEEP_DURATION: Duration = Duration::from_millis(5000);

#[derive(Debug)]
pub struct BrupopAgent {}

#[derive(Debug, PartialEq, Default)]
struct BottlerocketNodeMetadata {
    current_version: String,
}

impl BrupopAgent {
    pub fn new() -> BrupopAgent {
        BrupopAgent {}
    }

    pub async fn ensure_node_custom_resource_exists(&self) {}

    async fn gather_system_metadata(&self) -> BottlerocketNodeMetadata {
        // TODO(seankell) unimplemented
        Default::default()
    }

    /// Requests the metadata for the current BottlerocketNode from the Kubernetes custom resource associated
    /// with this node.
    async fn fetch_metadata_custom_resource(&self) -> BottlerocketNodeMetadata {
        // TODO(seankell) unimplemented
        Default::default()
    }

    async fn update_metadata_custom_resource(&self, _current_metadata: BottlerocketNodeMetadata) {
        // TODO
        unimplemented!()
    }

    pub async fn run(&mut self) {
        // A running agent has two responsibilities:
        // - Gather metadata about the system and update the custom resource associated with this node
        // - Determine if the spec on the system's custom resource demands the node take action. If so, begin taking that action.

        loop {
            let current_system_metadata = self.gather_system_metadata().await;
            let stored_system_metadata = self.fetch_metadata_custom_resource().await;

            if current_system_metadata != stored_system_metadata {
                // Update the system metadata
                log::info!("Detected drift between stored state and current state. Requesting to update node custom resource: {:?}.", &current_system_metadata);
                self.update_metadata_custom_resource(current_system_metadata)
                    .await;
            } else {
                log::info!("Did not detect updates to current system metadata.");
            }
            // Determine if the metadata has changed

            log::debug!(
                "Agent loop completed. Sleeping for {:?}.",
                AGENT_SLEEP_DURATION
            );
            sleep(AGENT_SLEEP_DURATION).await;
        }
    }
}
