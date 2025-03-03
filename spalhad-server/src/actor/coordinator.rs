use anyhow::Result;
use futures::{StreamExt, TryStreamExt, stream};
use spalhad_actor::{ActorCall, ActorHandle, CallSuperset, TrivialLoopActor};
use spalhad_spec::kv::Key;
use tokio::pin;

use super::storage::{self, StorageHandle};

#[derive(Debug)]
pub struct Coordinator {
    replication: usize,
    concurrency_level: usize,
    storage_table: Box<[StorageHandle]>,
}

impl Coordinator {
    pub fn new(
        replication: usize,
        concurrency_level: usize,
        nodes: impl IntoIterator<Item = StorageHandle>,
    ) -> Self {
        Self {
            replication,
            concurrency_level,
            storage_table: nodes.into_iter().collect(),
        }
    }
}

impl TrivialLoopActor for Coordinator {
    type Call = CoordinatorCall;

    async fn on_call(&mut self, call: Self::Call) -> Result<()> {
        match call {
            CoordinatorCall::Get(call) => {
                let i = call.input.key.partition(self.storage_table.len());
                let mut replicators = 0 .. self.replication;
                loop {
                    let Some(j) = replicators.next() else {
                        call.back.reply_ok(None);
                        break;
                    };
                    let get_message =
                        storage::Get { key: call.input.key.clone() };
                    let index = (i + j) % self.storage_table.len();
                    let output =
                        self.storage_table[index].send(get_message).await?;
                    if output.is_some() {
                        call.back.reply_ok(output);
                        break;
                    }
                }
            },

            CoordinatorCall::Put(call) => {
                let i = call.input.key.partition(self.storage_table.len());
                let nodes = &self.storage_table;
                let replication = self.replication;
                let concurrency_level = self.concurrency_level;

                call.handle(|input| async move {
                    let input = &input;
                    let task_stream = stream::iter(0 .. replication)
                        .map(|j| async move {
                            let index = (i + j) % nodes.len();
                            let put_message = storage::Put {
                                key: input.key.clone(),
                                value: input.value.clone(),
                            };
                            nodes[index].send(put_message).await
                        })
                        .buffer_unordered(concurrency_level);

                    pin!(task_stream);
                    let mut new_count = 0;
                    while let Some(new) = task_stream.try_next().await? {
                        if new {
                            new_count += 1;
                        }
                    }
                    Ok(new_count > replication / 2)
                })
                .await;
            },
        }

        Ok(())
    }
}

pub type CoordinatorHandle = ActorHandle<CoordinatorCall>;

#[derive(Debug, CallSuperset)]
pub enum CoordinatorCall {
    Get(GetCall),
    Put(PutCall),
}

#[derive(Debug, Clone)]
pub struct Get {
    pub key: Key,
}

pub type GetOutput = Option<serde_json::Value>;

pub type GetCall = ActorCall<Get, GetOutput>;

#[derive(Debug, Clone)]
pub struct Put {
    pub key: Key,
    pub value: serde_json::Value,
}

pub type PutOutput = bool;

pub type PutCall = ActorCall<Put, PutOutput>;
