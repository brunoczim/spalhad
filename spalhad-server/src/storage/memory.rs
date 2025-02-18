use std::collections::HashMap;

use anyhow::anyhow;
use tokio::{select, sync::mpsc};

use crate::taks::TaskManager;

use super::{StorageHandle, StorageMessage};

pub fn start(task_manager: &TaskManager, buffer_size: usize) -> StorageHandle {
    let (sender, mut receiver) = mpsc::channel(buffer_size);
    let handle = StorageHandle { channel: sender };
    let cancellation_token = task_manager.cancellation_token();
    let mut map = HashMap::new();

    task_manager.spawn(async move {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = receiver.recv() => message,
            };
            let message = result.ok_or_else(|| {
                anyhow!("memory database's sender disconnected")
            })?;
            match message {
                StorageMessage::Get(key, callback) => {
                    let value = map.get(&key).cloned();
                    callback
                        .send(value)
                        .map_err(|_| anyhow!("callback disconnected"))?;
                },
                StorageMessage::Put(key, value, callback) => {
                    let new = map.insert(key, value).is_none();
                    callback
                        .send(new)
                        .map_err(|_| anyhow!("callback disconnected"))?;
                },
            }
        }
    });

    handle
}
