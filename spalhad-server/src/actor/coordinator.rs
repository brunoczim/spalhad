use std::collections::HashMap;

use anyhow::{Result, anyhow, bail};
use futures::{StreamExt, stream};
use spalhad_actor::{ActorCall, ActorHandle, CallSuperset, TrivialLoopActor};
use spalhad_spec::kv::Key;
use tokio::pin;

use super::storage::{self, StorageHandle};

#[derive(Debug)]
pub struct Coordinator {
    replication: usize,
    min_correct_reads: usize,
    min_correct_writes: usize,
    concurrency_level: usize,
    storage_table: Box<[StorageHandle]>,
}

impl Coordinator {
    pub fn new(
        replication: usize,
        min_correct_reads: usize,
        min_correct_writes: usize,
        concurrency_level: usize,
        nodes: impl IntoIterator<Item = StorageHandle>,
    ) -> Self {
        Self {
            replication,
            min_correct_reads,
            min_correct_writes,
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
                tracing::trace!(
                    key = call.input.key.to_string(),
                    "handling get coordinator request",
                );
                let i = call.input.key.partition(self.storage_table.len());
                let replicators = 0 .. self.replication;
                let mut reads = HashMap::new();

                for j in replicators {
                    let get_message =
                        storage::Get { key: call.input.key.clone() };
                    let index = (i + j) % self.storage_table.len();
                    tracing::trace!(node = index, "asking node");
                    let output =
                        self.storage_table[index].send(get_message).await;
                    if let Ok(data) = output {
                        let count: &mut usize = reads.entry(data).or_default();
                        *count += 1;
                        if *count >= self.min_correct_reads {
                            break;
                        }
                    }
                }

                let mut answer = None;
                for (data, count) in reads {
                    let has_more_votes =
                        answer.as_ref().is_none_or(|(_, best)| count > *best);
                    if has_more_votes && count >= self.min_correct_reads {
                        answer = Some((data, count));
                    }
                }

                match answer {
                    Some((data, _)) => call.back.reply_ok(data),
                    None => {
                        call.reply_error(anyhow!("Failed to get consensus"))
                    },
                };
            },

            CoordinatorCall::Put(call) => {
                tracing::trace!(
                    key = call.input.key.to_string(),
                    "handling put coordinator request",
                );
                let i = call.input.key.partition(self.storage_table.len());
                let nodes = &self.storage_table;
                let replication = self.replication;
                let min_correct_writes = self.min_correct_writes;
                let concurrency_level = self.concurrency_level;

                call.handle(|input| async move {
                    let input = &input;
                    let task_stream = stream::iter(0 .. replication)
                        .map(|j| async move {
                            let index = (i + j) % nodes.len();
                            tracing::trace!(node = index, "sending to node");
                            let put_message = storage::Put {
                                key: input.key.clone(),
                                value: input.value.clone(),
                            };
                            nodes[index].send(put_message).await
                        })
                        .buffer_unordered(concurrency_level);

                    pin!(task_stream);
                    let mut answers = [0; 2];
                    while let Some(result) = task_stream.next().await {
                        if let Ok(new) = result {
                            answers[usize::from(new)] += 1;
                        }
                    }

                    let mut answer = None;
                    for (i, candidate) in answers.into_iter().enumerate() {
                        let has_more_votes =
                            answer.is_none_or(|best| candidate > answers[best]);
                        if has_more_votes && candidate >= min_correct_writes {
                            answer = Some(i);
                        }
                    }

                    match answer {
                        Some(i) => Ok(i != 0),
                        None => bail!("Failed to get consensus"),
                    }
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
