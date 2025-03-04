use spalhad_actor::{ActorCall, ActorHandle, CallSuperset};
use spalhad_spec::kv::Key;

pub use client::ClientStorage;
pub use dir::DirStorage;
pub use memory::MemoryStorage;

mod memory;
mod dir;
mod client;

pub type StorageHandle = ActorHandle<StorageCall>;

#[derive(Debug, CallSuperset)]
pub enum StorageCall {
    Get(GetCall),
    Put(PutCall),
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
