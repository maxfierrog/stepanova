//! # Device Client
//!
//! Entry point of the Stepanova client that runs on the reMarkable Paper Pro.

use anyhow::Result;

/* PROGRAM ENTRY */

fn main() -> Result<()>
{
    println!("stepanova {}", stepanova_core::VERSION);
    Ok(())
}
