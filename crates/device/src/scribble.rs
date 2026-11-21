//! # Scribble Module
//!
//! A minimal drawing surface for measuring ink latency through the takeover
//! display path: pen strokes are painted into the framebuffer and refreshed
//! with the fastest waveform as soon as they arrive. Exits on the power
//! button, a five-finger touch, or a termination signal, and reports the
//! software part of the latency (pen event timestamp to refresh submission).

use anyhow::Result;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;

use crate::display::Area;
use crate::display::Content;
use crate::display::Display;
use crate::display::Ink;
use crate::display::Mode;
use crate::input::Event;
use crate::input::Sample;
use crate::session::Session;

/* CONSTANTS */

const THRESHOLD: i32 = 40;
const ERASER: i32 = 22;
const FLUSH: Duration = Duration::from_millis(8);
const POLL: Duration = Duration::from_millis(5);

/* API STRUCTURES */

/// Drawing state for one run.
pub(crate) struct Scribble {
    latencies: Vec<Duration>,
    oldest: Option<SystemTime>,
    last: Option<(i32, i32)>,
    display: Display,
    flushed: Instant,
    dirty: Area,
}

/* IMPLEMENTATIONS */

impl Scribble {
    /// Clears the panel and prepares to draw.
    pub(crate) fn new(mut display: Display) -> Self {
        display.fill(Ink::White);
        let screen = display.screen();
        display.refresh(screen, Mode::Content, Content::Mono, true);
        Self {
            flushed: Instant::now(),
            dirty: Area::default(),
            latencies: Vec::new(),
            oldest: None,
            last: None,
            display,
        }
    }

    /// Draws until asked to stop, then prints latency statistics.
    pub(crate) fn run(
        mut self,
        session: &Session,
        events: Receiver<Event>,
    ) -> Result<()> {
        while !session.stopping() {
            match events.recv_timeout(POLL) {
                Ok(Event::Pen(sample)) => self.pen(sample),
                Ok(Event::Fingers(n)) if n >= 5 => break,
                Ok(Event::Power) => break,
                Ok(Event::Fingers(_)) => {},
                Err(RecvTimeoutError::Timeout) => {},
                Err(RecvTimeoutError::Disconnected) => break,
            }
            self.flush();
            self.display.pump();
        }
        self.report();
        Ok(())
    }
}

/* Utilities for Scribble */

impl Scribble {
    fn pen(&mut self, sample: Sample) {
        if !sample.touching || sample.pressure <= THRESHOLD {
            self.last = None;
            return;
        }
        let (ink, radius) = match sample.eraser {
            true => (Ink::White, ERASER),
            false => (Ink::Black, 2 + sample.pressure * 3 / 4096),
        };
        let here = (sample.x, sample.y);
        let area = match self.last {
            Some(from) => self
                .display
                .line(from, here, radius, ink),
            None => self
                .display
                .disc(here.0, here.1, radius, ink),
        };
        self.dirty = self.dirty.join(area);
        self.oldest = self.oldest.or(Some(sample.time));
        self.last = Some(here);
    }

    fn flush(&mut self) {
        if self.dirty.empty() || self.flushed.elapsed() < FLUSH {
            return;
        }
        let dirty = std::mem::take(&mut self.dirty);
        self.display
            .refresh(dirty, Mode::Pen, Content::Mono, false);
        if let Some(lag) = self
            .oldest
            .take()
            .and_then(|t| t.elapsed().ok())
        {
            self.latencies.push(lag);
        }
        self.flushed = Instant::now();
    }

    fn report(&mut self) {
        if self.latencies.is_empty() {
            eprintln!("stepanova: no strokes drawn");
            return;
        }
        self.latencies.sort();
        let at =
            |q: usize| self.latencies[(self.latencies.len() - 1) * q / 100];
        eprintln!(
            "stepanova: {} refreshes, event-to-submit p50 {:?} p95 {:?} max {:?}",
            self.latencies.len(),
            at(50),
            at(95),
            at(100),
        );
    }
}
