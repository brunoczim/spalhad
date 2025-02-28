use crate::actor::core::ActorHandle;

pub use client::ClientStorage;
pub use cluster::ClusterStorage;
pub use dir::DirStorage;
pub use memory::MemoryStorage;
use spalhad_spec::kv::Key;

use super::core::{ActorCall, CallSuperSet};

mod memory;
mod dir;
mod client;
mod cluster;

pub type StorageHandle = ActorHandle<StorageCall>;

#[derive(Debug)]
pub enum StorageCall {
    Get(GetCall),
    Put(PutCall),
}

impl From<GetCall> for StorageCall {
    fn from(message: GetCall) -> Self {
        Self::Get(message)
    }
}

impl From<PutCall> for StorageCall {
    fn from(message: PutCall) -> Self {
        Self::Put(message)
    }
}

impl CallSuperSet for StorageCall {
    fn reply_error<E>(self, error: E) -> bool
    where
        E: Into<anyhow::Error>,
    {
        match self {
            Self::Get(call) => call.reply_error(error),
            Self::Put(call) => call.reply_error(error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Get {
    pub key: Key,
}

pub type GetOutput = Option<serde_json::Value>;

pub type GetCall = ActorCall<Get, GetOutput>;

#[derive(Debug, Clone)]
pub struct Put {
    pub key: Key,
    pub value: serde_json::Value,
}

pub type PutOutput = bool;

pub type PutCall = ActorCall<Put, PutOutput>;
