//! # Server
//!
//! Entry point of the self-hosted Stepanova server.

use anyhow::Result;

fn main() -> Result<()> {
    println!("stepanova-server {}", stepanova_core::VERSION);
    Ok(())
}
