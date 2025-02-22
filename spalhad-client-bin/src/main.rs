use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use spalhad_client::Client;

#[derive(Debug, Clone, Parser)]
struct CliArgs {
    #[clap(short, long, default_value = "http://localhost:5500/kv")]
    base_url: String,
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Clone, Subcommand)]
enum Cmd {
    Get {
        #[clap(short, long)]
        key: String,
    },
    Put {
        #[clap(short, long)]
        key: String,
        #[clap(short, long)]
        value: String,
    },
}

async fn try_main(args: CliArgs) -> Result<()> {
    let client = Client::new(args.base_url);
    match args.cmd {
        Cmd::Get { key } => match client.get(key).await? {
            Some(value) => {
                let value: serde_json::Value = value;
                eprintln!("{}", serde_json::to_string_pretty(&value)?)
            },
            None => {
                bail!("Not found")
            },
        },
        Cmd::Put { key, value } => {
            let value: serde_json::Value = serde_json::from_str(&value)?;
            if client.put(key, value).await? {
                eprintln!("Inserted new entry");
            } else {
                eprintln!("Updated");
            }
        },
    }
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
        eprintln!("Backtrace:");
        eprintln!("{}", error.backtrace());
    }
}
