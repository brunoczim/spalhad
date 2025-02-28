use crate::actor::{
    core::ActorHandle,
    message::kv::{GetCall, PutCall},
};

pub use client::ClientStorage;
pub use cluster::ClusterStorage;
pub use dir::DirStorage;
pub use memory::MemoryStorage;

use super::core::CallSuperSet;

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
            Self::Get(call) => call.back.reply_error(error),
            Self::Put(call) => call.back.reply_error(error),
        }
    }
}
