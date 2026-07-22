//! bkg-p2p - Decentralized P2P AI Agent Network
//!
//! One binary. Distributed intelligence. Token-powered autonomy.

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use bkg_p2p::bootstrap;
use bkg_p2p::cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .bkg_p2p/.env if present
    bootstrap::load_env();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Silence llama.cpp/ggml logs unless --debug is passed
    if !cli.debug {
        bkg_p2p::inference::silence_llama_logs();
    }

    // For interactive mode, use minimal logging
    let log_level = match &cli.command {
        None | Some(Command::Start) | Some(Command::Chat(_)) | Some(Command::Run(_)) => {
            "bkg_p2p=warn"
        }
        _ => "bkg_p2p=info",
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
            bkg_p2p::cli::start::run_interactive().await?;
        }
        Some(Command::Chat(args)) => {
            bkg_p2p::cli::chat::run(args).await?;
        }
        // Ollama/vLLM-style commands
        Some(Command::Run(args)) => {
            bkg_p2p::cli::run::run(args).await?;
        }
        Some(Command::Pull(args)) => {
            bkg_p2p::cli::run::pull(args).await?;
        }
        Some(Command::List) => {
            bkg_p2p::cli::run::list().await?;
        }
        Some(Command::Ps) => {
            bkg_p2p::cli::run::ps().await?;
        }
        Some(Command::Models(args)) => {
            bkg_p2p::cli::models::run(args).await?;
        }
        Some(Command::Peers(args)) => {
            bkg_p2p::cli::peers::run(args).await?;
        }
        Some(Command::Serve(args)) => {
            bkg_p2p::cli::serve::run(args).await?;
        }
        Some(Command::Agent { cmd }) => {
            bkg_p2p::cli::agent::run(cmd).await?;
        }
        Some(Command::Network { cmd }) => {
            bkg_p2p::cli::network::run(cmd).await?;
        }
        Some(Command::Wallet { cmd }) => {
            bkg_p2p::cli::wallet::run(cmd).await?;
        }
        Some(Command::Tool { cmd }) => {
            bkg_p2p::cli::tool::run(cmd).await?;
        }
        Some(Command::Skill { cmd }) => {
            bkg_p2p::cli::skill::run(cmd).await?;
        }
        Some(Command::Vector(args)) => {
            bkg_p2p::cli::vector::run(args).await?;
        }
        Some(Command::Job(args)) => {
            bkg_p2p::cli::job::run(args).await?;
        }
        Some(Command::Test(args)) => {
            bkg_p2p::cli::test::run(args).await?;
        }
        Some(Command::Doctor) => {
            bkg_p2p::cli::doctor::run().await?;
        }
        Some(Command::Version) => {
            println!("bkg-p2p {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
