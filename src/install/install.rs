use anyhow::Result;

use super::claude;

pub async fn run(agent: Option<String>) -> Result<()> {
    let agent = agent.unwrap_or_else(|| "all".to_string());

    match agent.as_str() {
        "claude" => claude::install().await,
        "codex" => {
            println!("Codex hook installation not yet implemented");
            Ok(())
        }
        "gemini" => {
            println!("Gemini hook installation not yet implemented");
            Ok(())
        }
        "all" => {
            println!("Installing hooks for all detected agents...");
            println!();

            // Claude Code
            if dirs::home_dir().map(|h| h.join(".claude").exists()).unwrap_or(false) {
                println!("[Claude Code]");
                claude::install().await?;
                println!();
            }

            // TODO: Add codex and gemini when implemented
            Ok(())
        }
        other => {
            anyhow::bail!("Unknown agent: {}. Supported: claude, codex, gemini, all", other);
        }
    }
}
