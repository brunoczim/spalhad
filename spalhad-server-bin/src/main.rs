use std::{backtrace::BacktraceStatus, iter, path::PathBuf};

use anyhow::{Result, bail};
use clap::Parser;
use spalhad_actor::ActorOptions;
use spalhad_server::{
    actor::{
        coordinator::Coordinator,
        storage::{
            ClientStorage,
            DirStorage,
            MemoryStorage,
        },
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
    #[clap(long, default_value_t = 4)]
    concurrency_level: usize,
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

    let clients_low = cluster_config.addresses[.. args.self_id].iter();
    let clients_high = cluster_config.addresses[args.self_id + 1 ..].iter();
    let spawn_client =
        |base_url| storage_options.spawn(ClientStorage::open(base_url));

    let nodes = clients_low
        .map(spawn_client)
        .chain(iter::once(self_kv.clone()))
        .chain(clients_high.map(spawn_client));

    let coordinator = storage_options.spawn(Coordinator::new(
        cluster_config.replication,
        args.concurrency_level,
        nodes,
    ));

    let app = App::new(&storage_options, self_kv, coordinator)?;

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
