use anyhow::Result;
use spalhad_spec::Key;
use tokio::sync::{mpsc, oneshot};

pub mod memory;

#[derive(Debug)]
enum StorageMessage {
    Get(Key, oneshot::Sender<Option<serde_json::Value>>),
    Put(Key, serde_json::Value, oneshot::Sender<bool>),
}

#[derive(Debug, Clone)]
pub struct StorageHandle {
    channel: mpsc::Sender<StorageMessage>,
}

impl StorageHandle {
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
