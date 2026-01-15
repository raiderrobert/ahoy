use std::path::PathBuf;

/// Get the ahoy home directory (~/.ahoy)
pub fn home_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".ahoy")
}

/// Get the bin directory (~/.ahoy/bin)
pub fn bin_dir() -> PathBuf {
    home_dir().join("bin")
}
