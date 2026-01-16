use anyhow::Result;

use super::claude;

pub fn run(agent: Option<String>) -> Result<()> {
    let agent = agent.unwrap_or_else(|| "all".to_string());

    match agent.as_str() {
        "claude" => claude::uninstall(),
        "codex" => {
            println!("Codex hook uninstall not yet implemented");
            Ok(())
        }
        "gemini" => {
            println!("Gemini hook uninstall not yet implemented");
            Ok(())
        }
        "all" => {
            println!("Uninstalling hooks from all agents...");
            println!();

            // Claude Code
            println!("[Claude Code]");
            claude::uninstall()?;
            println!();

            // TODO: Add codex and gemini when implemented
            Ok(())
        }
        other => {
            anyhow::bail!("Unknown agent: {}. Supported: claude, codex, gemini, all", other);
        }
    }
}
