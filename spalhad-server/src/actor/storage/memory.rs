use std::collections::HashMap;

use anyhow::Result;
use spalhad_spec::kv::Key;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::actor::core::{Actor, ActorInbox};

use super::StorageCall;

#[derive(Debug, Clone)]
pub struct MemoryStorage {
    map: HashMap<Key, serde_json::Value>,
}

impl MemoryStorage {
    pub fn open() -> Self {
        Self { map: HashMap::new() }
    }
}

impl Actor for MemoryStorage {
    type Call = StorageCall;

    async fn start(
        mut self,
        mut inbox: ActorInbox<Self::Call>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = inbox.recv() => message,
            };
            let Some(call) = result else { break Ok(()) };

            match call {
                StorageCall::Get(call) => {
                    let map = &self.map;
                    call.handle(|input| async move {
                        Ok(map.get(&input.key).cloned())
                    })
                    .await;
                },

                StorageCall::Put(call) => {
                    call.handle(|input| async {
                        Ok(self.map.insert(input.key, input.value).is_none())
                    })
                    .await;
                },
            }
        }
    }
}
