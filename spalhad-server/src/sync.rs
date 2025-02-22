use anyhow::{Result, bail};
use spalhad_client::Client;
use spalhad_spec::cluster::RunId;

pub async fn check_self_address(
    self_run_id: RunId,
    self_base_url: &str,
) -> Result<()> {
    tracing::info!(%self_run_id, %self_base_url, "Testing correctness of self's base URL...");
    let client = Client::new(self_base_url);
    let responded_run_id = client.run_id().await?;
    tracing::info!(%responded_run_id, "Self's base URL test got a response");
    if responded_run_id != self_run_id {
        tracing::error!(%self_run_id, %responded_run_id, "Self's base URL is incorect");
        bail!(
            "The base URL {} given as self is not actually self",
            self_base_url
        );
    }
    tracing::info!("Self's base URL is correct");
    Ok(())
}
