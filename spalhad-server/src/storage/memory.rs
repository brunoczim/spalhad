use std::collections::HashMap;

use tokio::{select, sync::mpsc};

use crate::taks::TaskManager;

use super::StorageMessage;

pub fn start(
    task_manager: &TaskManager,
    mut receiver: mpsc::Receiver<StorageMessage>,
) {
    let cancellation_token = task_manager.cancellation_token();
    let mut map = HashMap::new();

    task_manager.spawn(async move {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = receiver.recv() => message,
            };
            let Some(message) = result else { break Ok(()) };
            match message {
                StorageMessage::Get(key, callback) => {
                    let value = map.get(&key).cloned();
                    if callback.send(value).is_err() {
                        break Ok(());
                    }
                },
                StorageMessage::Put(key, value, callback) => {
                    let new = map.insert(key, value).is_none();
                    if callback.send(new).is_err() {
                        break Ok(());
                    }
                },
            }
        }
    });
}
