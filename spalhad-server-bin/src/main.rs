use std::{backtrace::BacktraceStatus, iter, path::PathBuf};

use anyhow::{Result, bail};
use clap::Parser;
use spalhad_server::{
    http::{self, App},
    mux::Multiplexer,
    storage::StorageOptions,
    sync,
    taks::TaskManager,
};
use spalhad_spec::cluster::ClusterConfig;
use tokio::fs;
use tracing::Level;
use tracing_subscriber::{
    EnvFilter,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Debug, Clone, Parser)]
struct CliArgs {
    #[clap(short, long, default_value = "0.0.0.0:5500")]
    bind: String,
    #[clap(short, long, default_value_t = 10)]
    kv_channel_size: usize,
    #[clap(short, long)]
    persistence_dir: Option<PathBuf>,
    #[clap(short, long, default_value = "cluster.config.json")]
    cluster_config: PathBuf,
    #[clap(short, long)]
    self_id: usize,
}

fn setup_logging() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("SPALHAD_LOG_LEVEL")
        .from_env()?;
    let fmt = fmt::layer().with_target(true);
    tracing_subscriber::registry().with(fmt).with(env_filter).init();
    Ok(())
}

async fn try_main(args: CliArgs) -> Result<()> {
    setup_logging()?;

    let task_manager = TaskManager::new();

    let storage_options = StorageOptions::new(&task_manager)
        .with_channel_size(args.kv_channel_size);

    let self_kv = match args.persistence_dir {
        Some(dir_path) => storage_options.open_dir(dir_path),
        None => storage_options.open_memory(),
    };

    let cluster_config_contents = fs::read(&args.cluster_config).await?;
    let cluster_config: ClusterConfig =
        serde_json::from_slice(&cluster_config_contents)?;

    if args.self_id >= cluster_config.addresses.len() {
        bail!("self-id is too big")
    }

    tracing::info!("self-id is {}", args.self_id);

    let nodes = cluster_config.addresses[.. args.self_id]
        .iter()
        .map(|base_url| {
            StorageOptions::new(&task_manager)
                .with_channel_size(args.kv_channel_size)
                .open_client(base_url)
        })
        .chain(iter::once(self_kv))
        .chain(cluster_config.addresses[args.self_id + 1 ..].iter().map(
            |base_url| {
                StorageOptions::new(&task_manager)
                    .with_channel_size(args.kv_channel_size)
                    .open_client(base_url)
            },
        ));

    let app = App::new(Multiplexer::new(nodes))?;

    let self_run_id = app.self_run_id();
    let self_base_url = cluster_config.addresses[args.self_id].clone();

    let router = http::router().with_state(app);
    let bind_address = args.bind;
    task_manager.spawn(async move { http::serve(&bind_address, router).await });

    task_manager.spawn(async move {
        sync::check_self_address(self_run_id, &self_base_url).await
    });

    task_manager.wait_all().await?;
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

        if error.backtrace().status() == BacktraceStatus::Captured {
            eprintln!("Backtrace:");
            eprintln!("{}", error.backtrace());
        }
    }
}
