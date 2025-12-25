use anyhow::Result;
use spalhad_client::Client;
use spalhad_spec::cluster::RunId;

pub async fn activate(self_run_id: RunId, self_base_url: &str) -> Result<()> {
    tracing::info!(
        %self_run_id,
        %self_base_url,
        "Starting self activating task...",
    );
    let client = Client::new(self_base_url);
    tracing::info!("Sending request to activate self...");
    client.activate(self_run_id).await?;
    tracing::info!("Done. Active.");
    Ok(())
}
