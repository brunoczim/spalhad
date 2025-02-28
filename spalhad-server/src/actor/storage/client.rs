use anyhow::Result;
use spalhad_client::Client;

use crate::actor::core::TrivialLoopActor;

use super::StorageCall;

#[derive(Debug, Clone)]
pub struct ClientStorage {
    client: Client,
}

impl ClientStorage {
    pub fn open(base_url: impl Into<String>) -> Self {
        Self { client: Client::new(base_url.into()) }
    }
}

impl TrivialLoopActor for ClientStorage {
    type Call = StorageCall;

    async fn on_call(&mut self, call: Self::Call) -> Result<()> {
        match call {
            StorageCall::Get(call) => {
                call.handle(|input| async {
                    self.client.get_raw(input.key).await
                })
                .await;
            },

            StorageCall::Put(call) => {
                call.handle(|input| async {
                    self.client.put_raw(input.key, input.value).await
                })
                .await;
            },
        }

        Ok(())
    }
}
