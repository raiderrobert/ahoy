use std::path::PathBuf;

/// Get the ahoy home directory (~/.ahoy)
pub fn home_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".ahoy")
}

/// Get the socket path (~/.ahoy/ahoy.sock)
pub fn socket_path() -> PathBuf {
    home_dir().join("ahoy.sock")
}

/// Get the log file path (~/.ahoy/ahoy.log)
pub fn log_path() -> PathBuf {
    home_dir().join("ahoy.log")
}

/// Get the bin directory (~/.ahoy/bin)
pub fn bin_dir() -> PathBuf {
    home_dir().join("bin")
}

/// Ensure the ahoy home directory exists
pub fn ensure_home_dir() -> std::io::Result<()> {
    let home = home_dir();
    if !home.exists() {
        std::fs::create_dir_all(&home)?;
    }
    Ok(())
}
