use std::collections::HashMap;

use tokio::select;

use crate::taks::TaskManager;

use super::{StorageCall, StorageInbox};

pub fn start(task_manager: &TaskManager, mut inbox: StorageInbox) {
    let cancellation_token = task_manager.cancellation_token();
    let mut map = HashMap::new();

    task_manager.spawn(async move {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = inbox.recv() => message,
            };
            let Some(message) = result else { break Ok(()) };
            match message {
                StorageCall::Get(call) => {
                    let map = &map;
                    call.handle(|input| async move {
                        Ok(map.get(&input.key).cloned())
                    })
                    .await;
                },
                StorageCall::Put(call) => {
                    call.handle(|input| async {
                        Ok(map.insert(input.key, input.value).is_none())
                    })
                    .await;
                },
            }
        }
    });
}
