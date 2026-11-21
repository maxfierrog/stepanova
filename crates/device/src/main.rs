//! # Device Client
//!
//! Entry point of the Stepanova client that runs on the reMarkable Paper Pro.
//! It currently runs the scribble latency experiment through the takeover
//! display path.

mod display;
mod input;
mod scribble;
mod session;
mod vendor;

use anyhow::Result;

use display::Display;
use scribble::Scribble;
use session::Session;

fn main() -> Result<()> {
    eprintln!("stepanova {}", stepanova_core::VERSION);
    let session = Session::begin()?;
    let display = Display::open()?;
    let events = input::listen(display.width(), display.height())?;
    Scribble::new(display).run(&session, events)
}
