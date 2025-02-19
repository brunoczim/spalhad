use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use spalhad_server::{http, storage::StorageOptions, taks::TaskManager};
use tokio::try_join;
use tracing::Level;
use tracing_subscriber::{
    EnvFilter,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Debug, Clone, Parser)]
struct CliArgs {
    #[clap(short, long, default_value = "0.0.0.0:3000")]
    bind: String,
    #[clap(short, long, default_value_t = 10)]
    kv_channel_size: usize,
    #[clap(short, long)]
    persistence_dir: Option<PathBuf>,
}

fn setup_logging() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("SPALHAD_LOG_LEVEL")
        .from_env()?;
    let fmt = fmt::layer().with_target(false);
    tracing_subscriber::registry().with(fmt).with(env_filter).init();
    Ok(())
}

async fn try_main(args: CliArgs) -> Result<()> {
    setup_logging()?;

    let task_manager = TaskManager::new();

    let storage_options = StorageOptions::new(&task_manager)
        .with_channel_size(args.kv_channel_size);

    let kv = match args.persistence_dir {
        Some(dir_path) => storage_options.open_dir(dir_path),
        None => storage_options.open_memory(),
    };

    let router = http::router(kv);
    try_join!(http::serve(&args.bind, router), task_manager.wait_all())?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    if let Err(error) = try_main(args).await {
        eprintln!("Fatal error");
        for error in error.chain() {
            eprintln!("Caused by:");
            eprintln!("  - {error}")
        }
    }
}
