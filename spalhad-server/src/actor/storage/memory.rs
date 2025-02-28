use std::collections::HashMap;

use anyhow::Result;
use spalhad_spec::kv::Key;

use crate::actor::core::TrivialLoopActor;

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
                call.handle(
                    |input| async move { Ok(map.get(&input.key).cloned()) },
                )
                .await;
            },

            StorageCall::Put(call) => {
                call.handle(|input| async {
                    Ok(self.map.insert(input.key, input.value).is_none())
                })
                .await;
            },
        }

        Ok(())
    }
}
