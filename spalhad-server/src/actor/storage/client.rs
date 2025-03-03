use anyhow::Result;
use spalhad_actor::TrivialLoopActor;
use spalhad_client::Client;

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
                    self.client.get_internal(input.key).await
                })
                .await;
            },

            StorageCall::Put(call) => {
                call.handle(|input| async {
                    self.client.put_internal(input.key, input.value).await
                })
                .await;
            },
        }

        Ok(())
    }
}
