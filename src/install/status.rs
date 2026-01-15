use anyhow::Result;

use super::claude;

pub async fn run() -> Result<()> {
    println!("Installed hooks:");
    println!();

    // Claude Code
    let claude_installed = claude::is_installed();
    let claude_marker = if claude_installed { "x" } else { " " };
    let claude_status = if claude_installed { "installed" } else { "not installed" };
    println!("  [{}] Claude Code ({})", claude_marker, claude_status);

    // Codex (placeholder)
    println!("  [ ] Codex (not yet supported)");

    // Gemini (placeholder)
    println!("  [ ] Gemini CLI (not yet supported)");

    Ok(())
}
