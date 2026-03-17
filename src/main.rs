use ahoy::client;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ahoy")]
#[command(about = "Native macOS notifications for LLM coding agents")]
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
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Send {
            message,
            title,
            json,
            from_claude,
            activate,
        } => {
            client::send::run(message, title, json, from_claude, activate)?;
        }
    }

    Ok(())
}
