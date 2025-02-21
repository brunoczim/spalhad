use std::sync::Arc;

use anyhow::Result;
use spalhad_spec::kv::Key;

use crate::storage::StorageHandle;

#[derive(Debug, Clone)]
pub struct Multiplexer {
    nodes: Arc<[StorageHandle]>,
}

impl Multiplexer {
    pub fn new(nodes: impl IntoIterator<Item = StorageHandle>) -> Self {
        Self { nodes: nodes.into_iter().collect() }
    }

    pub async fn get(&self, key: Key) -> Result<Option<serde_json::Value>> {
        let node = self.select(&key);
        node.get(key).await
    }

    pub async fn put(
        &self,
        key: Key,
        value: serde_json::Value,
    ) -> Result<bool> {
        let node = self.select(&key);
        node.put(key, value).await
    }

    fn select(&self, key: &Key) -> &StorageHandle {
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
