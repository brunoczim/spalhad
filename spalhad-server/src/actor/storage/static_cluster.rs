use anyhow::Result;
use spalhad_actor::TrivialLoopActor;
use spalhad_spec::kv::Key;

use super::{StorageCall, StorageHandle};

#[derive(Debug, Clone)]
pub struct StaticClusterStorage {
    nodes: Box<[StorageHandle]>,
}

impl StaticClusterStorage {
    pub fn open(nodes: impl IntoIterator<Item = StorageHandle>) -> Self {
        Self { nodes: nodes.into_iter().collect() }
    }

    fn select<'a>(&'a self, key: &Key) -> &'a StorageHandle {
        let index = key.partition(self.nodes.len());
        tracing::trace!("multiplexing to storage {index}");
        &self.nodes[index]
    }
}

impl TrivialLoopActor for StaticClusterStorage {
    type Call = StorageCall;

    async fn on_call(&mut self, call: Self::Call) -> Result<()> {
        match call {
            StorageCall::Get(call) => {
                self.select(&call.input.key).forward(call).await?;
            },

            StorageCall::Put(call) => {
                self.select(&call.input.key).forward(call).await?;
            },
        }

        Ok(())
    }
}
