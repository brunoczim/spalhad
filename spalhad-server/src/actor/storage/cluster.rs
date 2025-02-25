use spalhad_spec::kv::Key;
use tokio::select;

use crate::taks::TaskManager;

use super::{StorageCall, StorageHandle, StorageInbox};

fn select<'a>(nodes: &'a [StorageHandle], key: &Key) -> &'a StorageHandle {
    let total = nodes.len().to_le_bytes();
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

    &nodes[index]
}

pub fn start(
    task_manager: &TaskManager,
    mut inbox: StorageInbox,
    nodes: Box<[StorageHandle]>,
) {
    let cancellation_token = task_manager.cancellation_token();

    task_manager.spawn(async move {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = inbox.recv() => message,
            };
            let Some(message) = result else { break Ok(()) };
            match message {
                StorageCall::Get(call) => {
                    let node = select(&nodes, &call.input().key);
                    node.forward(call).await?;
                },
                StorageCall::Put(call) => {
                    let node = select(&nodes, &call.input().key);
                    node.forward(call).await?;
                },
            }
        }
    });
}
