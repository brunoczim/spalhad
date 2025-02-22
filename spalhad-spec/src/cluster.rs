use serde::{Deserialize, Serialize};

pub use run_id::RunId;

pub mod run_id;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ClusterConfig {
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunIdResponse {
    pub run_id: RunId,
}
