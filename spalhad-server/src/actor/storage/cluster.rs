use anyhow::Result;
use spalhad_spec::kv::Key;

use crate::actor::core::ReactiveActor;

use super::{StorageCall, StorageHandle};

#[derive(Debug, Clone)]
pub struct ClusterStorage {
    nodes: Box<[StorageHandle]>,
}

impl ClusterStorage {
    pub fn open(nodes: impl IntoIterator<Item = StorageHandle>) -> Self {
        Self { nodes: nodes.into_iter().collect() }
    }

    fn select<'a>(&'a self, key: &Key) -> &'a StorageHandle {
        let total = self.nodes.len().to_le_bytes();
        let mut divisor = [0; Key::SIZE];
        divisor[.. total.len()].copy_from_slice(&total);
        let mut quotient = [0; Key::SIZE];
        let mut remainder = [0; Key::SIZE];
        key.divide_le(&divisor, &mut quotient, &mut remainder);

        const INDEX_SIZE: usize = (usize::BITS as usize) / 8;
        let mut index_bytes = [0; INDEX_SIZE];
        index_bytes[..].copy_from_slice(&remainder[.. INDEX_SIZE]);
        let index = usize::from_le_bytes(index_bytes);

        tracing::trace!("multiplexing to storage {index}");

        &self.nodes[index]
    }
}

impl ReactiveActor for ClusterStorage {
    type ReactiveCall = StorageCall;

    async fn on_call(&mut self, call: Self::ReactiveCall) -> Result<()> {
        match call {
            StorageCall::Get(call) => {
                self.select(&call.input().key).forward(call).await?;
            },

            StorageCall::Put(call) => {
                self.select(&call.input().key).forward(call).await?;
            },
        }

        Ok(())
    }
}
