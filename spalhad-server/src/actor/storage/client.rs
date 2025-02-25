use spalhad_client::Client;
use tokio::select;

use crate::taks::TaskManager;

use super::{StorageCall, StorageInbox};

pub fn start(
    task_manager: &TaskManager,
    mut inbox: StorageInbox,
    base_url: String,
) {
    let cancellation_token = task_manager.cancellation_token();
    let client = Client::new(base_url);

    task_manager.spawn(async move {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = inbox.recv() => message,
            };
            let Some(message) = result else { break Ok(()) };

            match message {
                StorageCall::Get(call) => {
                    call.handle(|input| async {
                        client.get_raw(input.key).await
                    })
                    .await;
                },
                StorageCall::Put(call) => {
                    call.handle(|input| async {
                        client.put_raw(input.key, input.value).await
                    })
                    .await;
                },
            }
        }
    });
}
