use std::path::PathBuf;

use crate::{
    actor::{
        self,
        core::{ActorHandle, ActorInbox},
        message::kv::{GetCall, PutCall},
    },
    taks::TaskManager,
};

mod memory;
mod dir;
mod client;
mod switch;

pub type StorageHandle = ActorHandle<StorageCall>;

pub type StorageInbox = ActorInbox<StorageCall>;

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

#[derive(Debug, Clone)]
pub struct StorageOptions<'a> {
    task_manager: &'a TaskManager,
    channel_size: usize,
}

impl<'a> StorageOptions<'a> {
    pub fn new(task_manager: &'a TaskManager) -> Self {
        Self { task_manager, channel_size: 10 }
    }

    pub fn set_channel_size(&mut self, size: usize) -> &mut Self {
        self.channel_size = size;
        self
    }

    pub fn with_channel_size(mut self, size: usize) -> Self {
        self.set_channel_size(size);
        self
    }

    pub fn open_memory(&self) -> StorageHandle {
        let (handle, inbox) = actor::core::channel(self.channel_size);
        memory::start(self.task_manager, inbox);
        handle
    }

    pub fn open_dir(&self, dir_path: impl Into<PathBuf>) -> StorageHandle {
        let (handle, inbox) = actor::core::channel(self.channel_size);
        dir::start(self.task_manager, inbox, dir_path.into());
        handle
    }

    pub fn open_client(&self, base_url: impl Into<String>) -> StorageHandle {
        let (handle, inbox) = actor::core::channel(self.channel_size);
        client::start(self.task_manager, inbox, base_url.into());
        handle
    }

    pub fn open_switch(
        &self,
        nodes: impl IntoIterator<Item = StorageHandle>,
    ) -> StorageHandle {
        let (handle, inbox) = actor::core::channel(self.channel_size);
        switch::start(self.task_manager, inbox, nodes.into_iter().collect());
        handle
    }
}
