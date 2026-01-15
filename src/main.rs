// Embed Info.plist into binary for macOS bundle identity
// This allows UNUserNotificationCenter to work properly
#[cfg(target_os = "macos")]
embed_plist::embed_info_plist!("../Info.plist");

use clap::{Parser, Subcommand};

mod client;
mod config;
mod daemon;
mod install;
mod notify;
mod service;

#[derive(Parser)]
#[command(name = "ahoy")]
#[command(about = "Cross-platform notification daemon for LLM coding agents")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a notification
    Send {
        /// The notification message
        message: Option<String>,

        /// Notification title
        #[arg(short, long, default_value = "Ahoy")]
        title: String,

        /// Send raw JSON message
        #[arg(long)]
        json: Option<String>,

        /// Read Claude Code hook data from stdin to extract last prompt
        #[arg(long)]
        from_claude: bool,
    },

    /// Run the notification daemon
    Daemon,

    /// Check daemon status
    Status,

    /// Tail daemon logs
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value = "20")]
        lines: usize,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Install hooks for LLM CLI agents
    Install {
        /// Agent to install hook for (claude, codex, gemini)
        agent: Option<String>,

        /// Show installation status
        #[arg(long)]
        status: bool,
    },

    /// Remove hooks from LLM CLI agents
    Uninstall {
        /// Agent to uninstall hook from (claude, codex, gemini, or all)
        agent: Option<String>,
    },

    /// Manage the background daemon service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand)]
enum ServiceAction {
    /// Install the daemon as a system service (auto-start on login)
    Install,
    /// Uninstall the daemon service
    Uninstall,
    /// Start the daemon service
    Start,
    /// Stop the daemon service
    Stop,
    /// Restart the daemon service
    Restart,
    /// Show service status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Send { message, title, json, from_claude } => {
            client::send::run(message, title, json, from_claude).await?;
        }
        Commands::Daemon => {
            daemon::run().await?;
        }
        Commands::Status => {
            client::status::run().await?;
        }
        Commands::Logs { lines, follow } => {
            client::logs::run(lines, follow).await?;
        }
        Commands::Install { agent, status } => {
            if status {
                install::status::run().await?;
            } else {
                install::install::run(agent).await?;
            }
        }
        Commands::Uninstall { agent } => {
            install::uninstall::run(agent).await?;
        }
        Commands::Service { action } => {
            match action {
                ServiceAction::Install => service::install().await?,
                ServiceAction::Uninstall => service::uninstall().await?,
                ServiceAction::Start => service::start().await?,
                ServiceAction::Stop => service::stop().await?,
                ServiceAction::Restart => service::restart().await?,
                ServiceAction::Status => service::status().await?,
            }
        }
    }

    Ok(())
}
