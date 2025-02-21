use std::path::PathBuf;

use anyhow::Result;
use spalhad_spec::kv::Key;
use tokio::sync::{mpsc, oneshot};

use crate::taks::TaskManager;

mod memory;
mod dir;
mod client;

#[derive(Debug)]
enum StorageMessage {
    Get(Key, oneshot::Sender<Option<serde_json::Value>>),
    Put(Key, serde_json::Value, oneshot::Sender<bool>),
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
        let (handle, receiver) = StorageHandle::open(self.channel_size);
        memory::start(self.task_manager, receiver);
        handle
    }

    pub fn open_dir(&self, dir_path: impl Into<PathBuf>) -> StorageHandle {
        let (handle, receiver) = StorageHandle::open(self.channel_size);
        dir::start(self.task_manager, receiver, dir_path.into());
        handle
    }

    pub fn open_client(&self, base_url: impl Into<String>) -> StorageHandle {
        let (handle, receiver) = StorageHandle::open(self.channel_size);
        client::start(self.task_manager, receiver, base_url.into());
        handle
    }
}

#[derive(Debug, Clone)]
pub struct StorageHandle {
    channel: mpsc::Sender<StorageMessage>,
}

impl StorageHandle {
    fn open(channel_size: usize) -> (Self, mpsc::Receiver<StorageMessage>) {
        let (sender, receiver) = mpsc::channel(channel_size);
        let handle = Self { channel: sender };
        (handle, receiver)
    }

    pub async fn get(&self, key: Key) -> Result<Option<serde_json::Value>> {
        let (sender, receiver) = oneshot::channel();
        self.channel.send(StorageMessage::Get(key, sender)).await?;
        let value = receiver.await?;
        Ok(value)
    }

    pub async fn put(
        &self,
        key: Key,
        value: serde_json::Value,
    ) -> Result<bool> {
        let (sender, receiver) = oneshot::channel();
        self.channel.send(StorageMessage::Put(key, value, sender)).await?;
        let new = receiver.await?;
        Ok(new)
    }
}
