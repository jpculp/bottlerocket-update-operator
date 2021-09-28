use crate::models::node::BottlerocketNodeStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertBottlerocketNodeRequest {
    pub node_name: String,
    pub node_status: BottlerocketNodeStatus,
}
