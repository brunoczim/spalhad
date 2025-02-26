use anyhow::Result;
use spalhad_client::Client;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::actor::core::{Actor, ActorInbox};

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

impl Actor for ClientStorage {
    type Call = StorageCall;

    async fn start(
        self,
        mut inbox: ActorInbox<Self::Call>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = inbox.recv() => message,
            };
            let Some(call) = result else { break Ok(()) };

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
        }
    }
}
