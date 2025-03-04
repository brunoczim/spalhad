use serde::{Deserialize, Serialize};

use crate::random_id::RandomId;

pub type RunId = RandomId<16>;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ClusterConfig {
    pub replication: usize,
    pub min_correct_reads: usize,
    pub min_correct_writes: usize,
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunIdResponse {
    pub run_id: RunId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivateRequest {
    pub run_id: RunId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IsActiveResponse {
    pub is_active: bool,
}

pub type ActivateResponse = IsActiveResponse;
