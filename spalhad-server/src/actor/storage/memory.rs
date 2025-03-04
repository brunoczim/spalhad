use std::collections::HashMap;

use anyhow::Result;
use spalhad_actor::TrivialLoopActor;
use spalhad_spec::kv::Key;

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

impl TrivialLoopActor for MemoryStorage {
    type Call = StorageCall;

    async fn on_call(&mut self, call: Self::Call) -> Result<()> {
        match call {
            StorageCall::Get(call) => {
                let map = &self.map;
                call.handle(|input| async move {
                    tracing::trace!(
                        key = input.key.to_string(),
                        "handling get memory storage request",
                    );
                    Ok(map.get(&input.key).cloned())
                })
                .await;
            },

            StorageCall::Put(call) => {
                call.handle(|input| async {
                    tracing::trace!(
                        key = input.key.to_string(),
                        "handling put memory storage request",
                    );
                    Ok(self.map.insert(input.key, input.value).is_none())
                })
                .await;
            },
        }

        Ok(())
    }
}
