use std::{backtrace::BacktraceStatus, path::PathBuf, time::Duration};

use anyhow::{Result, bail};
use clap::Parser;
use spalhad_actor::ActorOptions;
use spalhad_server::{
    actor::{
        coordinator::Coordinator,
        storage::{ClientStorage, DirStorage, MemoryStorage},
    },
    http::{self, App},
    sync,
};
use spalhad_spec::cluster::ClusterConfig;
use spalhad_task::TaskManager;
use tokio::fs;
use tracing::Level;
use tracing_subscriber::{
    EnvFilter,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

mod util;

#[derive(Debug, Clone, Parser)]
struct CliArgs {
    #[clap(short, long, default_value = "0.0.0.0:5000")]
    bind: String,
    #[clap(short, long, default_value_t = 10)]
    kv_channel_size: usize,
    #[clap(short, long)]
    persistence_dir: Option<PathBuf>,
    #[clap(short, long, default_value = "cluster.config.json")]
    cluster_config: PathBuf,
    #[clap(long, default_value_t = 4)]
    concurrency_level: usize,
    #[clap(short, long)]
    self_id: usize,
    #[clap(
        short = 't',
        long,
        default_value = "200ms",
        value_parser = util::parse_duration,
    )]
    communication_timeout: Duration,
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

    let storage_options = ActorOptions::new(&task_manager)
        .with_channel_size(args.kv_channel_size);

    let self_kv = match args.persistence_dir {
        Some(dir_path) => storage_options.spawn(DirStorage::open(dir_path)),
        None => storage_options.spawn(MemoryStorage::open()),
    };

    let cluster_config_contents = fs::read(&args.cluster_config).await?;
    let cluster_config: ClusterConfig =
        serde_json::from_slice(&cluster_config_contents)?;

    if args.self_id >= cluster_config.addresses.len() {
        bail!("self-id is too big")
    }

    tracing::info!("self-id is {}", args.self_id);

    let mut nodes = Vec::with_capacity(cluster_config.addresses.len());
    for (i, address) in cluster_config.addresses.iter().enumerate() {
        if i == args.self_id {
            nodes.push(self_kv.clone());
        } else {
            let client_storage_actor = ClientStorage::open_with_timeout(
                address,
                args.communication_timeout,
            )?;
            nodes.push(storage_options.spawn(client_storage_actor));
        }
    }

    let coordinator = storage_options.spawn(Coordinator::new(
        cluster_config.replication,
        cluster_config.min_correct_reads,
        cluster_config.min_correct_writes,
        args.concurrency_level,
        nodes,
    ));

    let app = App::new(&storage_options, self_kv, coordinator);

    let self_run_id = app.self_run_id();
    let self_base_url = cluster_config.addresses[args.self_id].clone();

    let router = http::router().with_state(app);
    let bind_address = args.bind;
    task_manager.spawn(async move { http::serve(&bind_address, router).await });

    task_manager.spawn(async move {
        sync::activate(self_run_id, &self_base_url).await
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
