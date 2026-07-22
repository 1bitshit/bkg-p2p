//! bkg-peer - Decentralized P2P AI Agent Network
//!
//! One binary. Distributed intelligence. Token-powered autonomy.

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use bkg-peer::bootstrap;
use bkg-peer::cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .bkg-peer/.env if present
    bootstrap::load_env();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Silence llama.cpp/ggml logs unless --debug is passed
    if !cli.debug {
        bkg-peer::inference::silence_llama_logs();
    }

    // For interactive mode, use minimal logging
    let log_level = match &cli.command {
        None | Some(Command::Start) | Some(Command::Chat(_)) | Some(Command::Run(_)) => {
            "bkg-peer=warn"
        }
        _ => "bkg-peer=info",
    };

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Dispatch to command handlers
    match cli.command {
        // No command = interactive mode
        None | Some(Command::Start) => {
            bkg-peer::cli::start::run_interactive().await?;
        }
        Some(Command::Chat(args)) => {
            bkg-peer::cli::chat::run(args).await?;
        }
        // Ollama/vLLM-style commands
        Some(Command::Run(args)) => {
            bkg-peer::cli::run::run(args).await?;
        }
        Some(Command::Pull(args)) => {
            bkg-peer::cli::run::pull(args).await?;
        }
        Some(Command::List) => {
            bkg-peer::cli::run::list().await?;
        }
        Some(Command::Ps) => {
            bkg-peer::cli::run::ps().await?;
        }
        Some(Command::Models(args)) => {
            bkg-peer::cli::models::run(args).await?;
        }
        Some(Command::Peers(args)) => {
            bkg-peer::cli::peers::run(args).await?;
        }
        Some(Command::Serve(args)) => {
            bkg-peer::cli::serve::run(args).await?;
        }
        Some(Command::Agent { cmd }) => {
            bkg-peer::cli::agent::run(cmd).await?;
        }
        Some(Command::Network { cmd }) => {
            bkg-peer::cli::network::run(cmd).await?;
        }
        Some(Command::Wallet { cmd }) => {
            bkg-peer::cli::wallet::run(cmd).await?;
        }
        Some(Command::Tool { cmd }) => {
            bkg-peer::cli::tool::run(cmd).await?;
        }
        Some(Command::Skill { cmd }) => {
            bkg-peer::cli::skill::run(cmd).await?;
        }
        Some(Command::Vector(args)) => {
            bkg-peer::cli::vector::run(args).await?;
        }
        Some(Command::Job(args)) => {
            bkg-peer::cli::job::run(args).await?;
        }
        Some(Command::Test(args)) => {
            bkg-peer::cli::test::run(args).await?;
        }
        Some(Command::Doctor) => {
            bkg-peer::cli::doctor::run().await?;
        }
        Some(Command::Version) => {
            println!("bkg-peer {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
