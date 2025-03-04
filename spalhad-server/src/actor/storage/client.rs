use std::time::Duration;

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

    pub fn open_with_timeout(
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self> {
        Ok(Self { client: Client::with_timeout(base_url.into(), timeout)? })
    }
}

impl TrivialLoopActor for ClientStorage {
    type Call = StorageCall;

    async fn on_call(&mut self, call: Self::Call) -> Result<()> {
        match call {
            StorageCall::Get(call) => {
                call.handle(|input| async {
                    tracing::trace!(
                        key = input.key.to_string(),
                        "handling get client storage request",
                    );
                    self.client.get_internal(input.key).await
                })
                .await;
            },

            StorageCall::Put(call) => {
                call.handle(|input| async {
                    tracing::trace!(
                        key = input.key.to_string(),
                        "handling put client storage request",
                    );
                    self.client.put_internal(input.key, input.value).await
                })
                .await;
            },
        }

        Ok(())
    }
}
