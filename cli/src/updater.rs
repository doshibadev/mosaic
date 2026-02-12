use crate::logger::Logger;
use anyhow::{Result, anyhow};
use self_update::cargo_crate_version;

/// Checks if GitHub has a shiny new version for us.
///
/// Runs in the background because nobody likes waiting for network calls.
/// If there's an update, we nudge the user gently.
pub async fn check_for_updates() -> Result<()> {
    let current_version = cargo_crate_version!();
    
    // We wrap the synchronous update check in spawn_blocking because
    // blocking the async runtime for a network call is rude.
    let status = tokio::task::spawn_blocking(move || {
        self_update::backends::github::Update::configure()
            .repo_owner("doshibadev")
            .repo_name("mosaic")
            .bin_name("mosaic")
            .current_version(current_version)
            .build()
            .map(|u| u.get_latest_release())
    }).await??;

    if let Ok(latest) = status {
        let latest_version = latest.version;
        if latest_version != current_version {
            println!();
            Logger::warn(format!(
                "Update available! {} -> {}",
                current_version,
                Logger::highlight(&latest_version)
            ));
            println!("  Run {} to upgrade.", Logger::brand_text("mosaic upgrade"));
            println!();
        }
    }

    Ok(())
}

/// Downloads the latest binary and replaces the current executable.
///
/// Yes, it modifies the running binary. It's magic (and supported by the OS).
pub async fn upgrade() -> Result<()> {
    Logger::info("Checking for updates...");
    
    let current_version = cargo_crate_version!();
    
    let status = tokio::task::spawn_blocking(move || {
        self_update::backends::github::Update::configure()
            .repo_owner("doshibadev")
            .repo_name("mosaic")
            .bin_name("mosaic")
            .show_download_progress(true)
            .current_version(current_version)
            .build()
            .map(|u| u.update())
    }).await??;

    match status {
        Ok(status) => {
            if status.updated() {
                Logger::success(format!(
                    "Upgraded to version {}!",
                    Logger::highlight(status.version())
                ));
            } else {
                Logger::info("Already up to date.");
            }
        }
        Err(e) => {
            return Err(anyhow!("Update failed: {}", e));
        }
    }

    Ok(())
}
