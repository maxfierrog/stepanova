//! # Session Module
//!
//! Exclusive ownership of the panel. A [`Session`] stops the stock UI
//! (`xochitl`) and holds a wake lock so the device cannot autosuspend while it
//! is down; dropping the session restores both. The stock UI must always come
//! back, because it is what unlocks the encrypted disk after a reboot, so a
//! detached watchdog also restores it if this process dies without unwinding
//! (for example when the vendor engine aborts, or on `SIGKILL`).

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use signal_hook::consts::SIGINT;
use signal_hook::consts::SIGTERM;
use signal_hook::flag;

use std::fs;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/* CONSTANTS */

const UNLOCK: &str = "/sys/power/wake_unlock";
const LOCKFILES: [&str; 2] = ["/tmp/epframebuffer.lock", "/tmp/epd.lock"];
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
        watchdog().context("Failed to start the xochitl watchdog.")?;
        if let Err(e) = systemctl("stop") {
            eprintln!("stepanova: {e:#}");
        }
        LOCKFILES
            .iter()
            .for_each(|f| drop(fs::remove_file(f)));
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

fn watchdog() -> Result<()> {
    let script = format!(
        "while kill -0 {pid} 2>/dev/null; do sleep 1; done; \
         systemctl start xochitl; echo {NAME} > {UNLOCK}",
        pid = std::process::id(),
    );
    Command::new("sh")
        .args(["-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    Ok(())
}

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
