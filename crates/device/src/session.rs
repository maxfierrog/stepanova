//! # Session Module
//!
//! Exclusive ownership of the panel. A [`Session`] stops the stock UI
//! (`xochitl`) and holds a wake lock so the device cannot autosuspend while it
//! is down; dropping the session restores both. The stock UI must always come
//! back, because it is what unlocks the encrypted disk after a reboot.

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use signal_hook::consts::SIGINT;
use signal_hook::consts::SIGTERM;
use signal_hook::flag;

use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/* CONSTANTS */

const UNLOCK: &str = "/sys/power/wake_unlock";
const LOCKFILE: &str = "/tmp/epframebuffer.lock";
const LOCK: &str = "/sys/power/wake_lock";
const NAME: &str = "stepanova";

/* API STRUCTURES */

/// Takeover of the panel, released on drop.
pub(crate) struct Session {
    stop: Arc<AtomicBool>,
}

/* IMPLEMENTATIONS */

impl Session {
    /// Stops the stock UI and starts watching for termination signals.
    pub(crate) fn begin() -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        [SIGTERM, SIGINT]
            .into_iter()
            .try_for_each(|s| flag::register(s, stop.clone()).map(drop))
            .context("Failed to register signal handlers.")?;
        let _ = fs::write(LOCK, NAME);
        systemctl("stop")?;
        let _ = fs::remove_file(LOCKFILE);
        Ok(Self { stop })
    }

    /// Whether a termination signal has been received.
    pub(crate) fn stopping(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Err(e) = systemctl("start") {
            eprintln!("stepanova: {e:#}");
        }
        let _ = fs::write(UNLOCK, NAME);
    }
}

/* Utilities for Session */

fn systemctl(action: &str) -> Result<()> {
    let status = Command::new("systemctl")
        .args([action, "xochitl"])
        .status()
        .with_context(|| format!("Failed to run systemctl {action}."))?;
    if !status.success() {
        bail!("systemctl {action} xochitl exited with {status}.");
    }
    Ok(())
}
