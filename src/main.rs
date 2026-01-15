// Embed Info.plist into binary for macOS bundle identity
// This allows UNUserNotificationCenter to work properly
#[cfg(target_os = "macos")]
embed_plist::embed_info_plist!("../Info.plist");

use clap::{Parser, Subcommand};

mod client;
mod config;
mod install;
mod notify;

#[derive(Parser)]
#[command(name = "ahoy")]
#[command(about = "Cross-platform notification CLI for LLM coding agents")]
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

        /// Bundle ID to activate when notification is clicked
        #[arg(long)]
        activate: Option<String>,
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
        Commands::Send { message, title, json, from_claude, activate } => {
            client::send::run(message, title, json, from_claude, activate)?;
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
    }

    Ok(())
}
