use spalhad_client::Client;
use tokio::{select, sync::mpsc};

use crate::taks::TaskManager;

use super::StorageMessage;

pub fn start(
    task_manager: &TaskManager,
    mut receiver: mpsc::Receiver<StorageMessage>,
    base_url: String,
) {
    let cancellation_token = task_manager.cancellation_token();
    let client = Client::new(base_url);

    task_manager.spawn(async move {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = receiver.recv() => message,
            };
            let Some(message) = result else { break Ok(()) };
            match message {
                StorageMessage::Get(key, callback) => {
                    let value = client.get_raw(key).await?;
                    if callback.send(value).is_err() {
                        break Ok(());
                    }
                },
                StorageMessage::Put(key, value, callback) => {
                    let new = client.put_raw(key, value).await?;
                    if callback.send(new).is_err() {
                        break Ok(());
                    }
                },
            }
        }
    });
}
