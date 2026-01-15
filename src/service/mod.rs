#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

use anyhow::Result;

pub async fn install() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::install().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::install().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Service management not supported on this platform")
    }
}

pub async fn uninstall() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::uninstall().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::uninstall().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Service management not supported on this platform")
    }
}

pub async fn start() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::start().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::start().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Service management not supported on this platform")
    }
}

pub async fn stop() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::stop().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::stop().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Service management not supported on this platform")
    }
}

pub async fn restart() -> Result<()> {
    stop().await.ok(); // Ignore error if not running
    start().await
}

pub async fn status() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::status().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::status().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Service management not supported on this platform")
    }
}
