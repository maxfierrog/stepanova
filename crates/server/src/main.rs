//! # Server
//!
//! Entry point of the self-hosted Stepanova server.

use anyhow::Result;

/* PROGRAM ENTRY */

fn main() -> Result<()>
{
    println!("stepanova-server {}", stepanova_core::VERSION);
    Ok(())
}
