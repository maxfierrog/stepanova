//! # Display Module
//!
//! Safe access to the Paper Pro panel through the vendor e-paper engine:
//! a writable RGB32 framebuffer plus region refreshes with a waveform hint.
//! Using it requires the stock UI to be stopped (see [`crate::session`]).

use anyhow::Result;

use crate::vendor::Buffer;
use crate::vendor::Engine;
use crate::vendor::Rect;

/* CONSTANTS */

const WHITE: u32 = 0xffff_ffff;
const BLACK: u32 = 0xff00_0000;

/* ENUMERATIONS */

/// Waveform family used to refresh a region (the vendor's `EPScreenMode`).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    Pen = 0,
    Animation = 2,
    Mono = 1,
    Ui = 3,
    Content = 4,
}

/// Kind of content being refreshed (the vendor's `EPContentType`).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Content {
    Mono = 0,
    Gray = 1,
    Color = 2,
}

/// Pixel value to paint with.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Ink {
    Black,
    White,
}

/* API STRUCTURES */

/// The panel, owned by this process for its lifetime.
pub(crate) struct Display {
    buffer: Buffer,
    engine: Engine,
}

/// A dirty region that grows to cover everything painted into it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Area {
    bounds: Option<(i32, i32, i32, i32)>,
}

/* IMPLEMENTATIONS */

impl Display {
    /// Starts the vendor engine and takes over its framebuffer.
    pub(crate) fn open() -> Result<Self> {
        let engine = Engine::start()?;
        let buffer = engine.buffer();
        Ok(Self { buffer, engine })
    }

    /// Width in pixels.
    pub(crate) fn width(&self) -> i32 {
        self.buffer.width as i32
    }

    /// Height in pixels.
    pub(crate) fn height(&self) -> i32 {
        self.buffer.height as i32
    }

    /// Paints the whole framebuffer without refreshing the panel.
    pub(crate) fn fill(&mut self, ink: Ink) {
        let value = ink.value();
        (0..self.height()).for_each(|y| self.row(y).fill(value));
    }

    /// Paints a filled disc of radius `r` centred on `(x, y)`.
    pub(crate) fn disc(&mut self, x: i32, y: i32, r: i32, ink: Ink) -> Area {
        let value = ink.value();
        let top = (y - r).max(0);
        let bottom = (y + r).min(self.height() - 1);
        for row in top..=bottom {
            let dy = row - y;
            let span = isqrt(r * r - dy * dy);
            let left = (x - span).max(0) as usize;
            let right = (x + span).min(self.width() - 1) as usize;
            if left <= right {
                self.row(row)[left..=right].fill(value);
            }
        }
        Area::around(x, y, r)
    }

    /// Paints a stroke of radius `r` from `a` to `b`.
    pub(crate) fn line(
        &mut self,
        a: (i32, i32),
        b: (i32, i32),
        r: i32,
        ink: Ink,
    ) -> Area {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let steps = dx.abs().max(dy.abs()).max(1);
        (0..=steps)
            .map(|i| (a.0 + dx * i / steps, a.1 + dy * i / steps))
            .fold(Area::default(), |area, (x, y)| {
                area.join(self.disc(x, y, r, ink))
            })
    }

    /// Refreshes `area` on the panel. Returns the vendor's update token, or
    /// zero when nothing was submitted.
    pub(crate) fn refresh(
        &self,
        area: Area,
        mode: Mode,
        content: Content,
        full: bool,
    ) -> u64 {
        let clipped = area.clip(self.width(), self.height());
        let Some((x0, y0, x1, y1)) = clipped.bounds else {
            return 0;
        };
        let rect = Rect::new(x0, y0, x1 - x0 + 1, y1 - y0 + 1);
        self.engine
            .swap(rect, content as i32, mode as i32, full)
    }

    /// Area covering the whole panel.
    pub(crate) fn screen(&self) -> Area {
        Area {
            bounds: Some((0, 0, self.width() - 1, self.height() - 1)),
        }
    }

    /// Runs pending vendor engine work; call once per loop iteration.
    pub(crate) fn pump(&self) {
        self.engine.pump();
    }
}

impl Area {
    /// Whether nothing has been painted into this area.
    pub(crate) fn empty(&self) -> bool {
        self.bounds.is_none()
    }

    /// Smallest area covering both `self` and `other`.
    pub(crate) fn join(self, other: Area) -> Area {
        let bounds = match (self.bounds, other.bounds) {
            (Some(a), Some(b)) => Some((
                a.0.min(b.0),
                a.1.min(b.1),
                a.2.max(b.2),
                a.3.max(b.3),
            )),
            (a, b) => a.or(b),
        };
        Area { bounds }
    }
}

/* Utilities for Display */

impl Display {
    fn row(&mut self, y: i32) -> &mut [u32] {
        let offset = y as usize * self.buffer.stride;
        let width = self.buffer.width;
        unsafe {
            let start = self
                .buffer
                .pixels
                .add(offset)
                .cast::<u32>();
            std::slice::from_raw_parts_mut(start, width)
        }
    }
}

impl Area {
    fn around(x: i32, y: i32, r: i32) -> Area {
        Area {
            bounds: Some((x - r, y - r, x + r, y + r)),
        }
    }

    fn clip(self, width: i32, height: i32) -> Area {
        let bounds = self
            .bounds
            .and_then(|(x0, y0, x1, y1)| {
                let (x0, y0) = (x0.max(0), y0.max(0));
                let (x1, y1) = (x1.min(width - 1), y1.min(height - 1));
                (x0 <= x1 && y0 <= y1).then_some((x0, y0, x1, y1))
            });
        Area { bounds }
    }
}

impl Ink {
    fn value(self) -> u32 {
        match self {
            Ink::Black => BLACK,
            Ink::White => WHITE,
        }
    }
}

fn isqrt(n: i32) -> i32 {
    (n.max(0) as f64).sqrt() as i32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn join_covers_both() {
        let a = Area::around(10, 10, 2);
        let b = Area::around(20, 5, 1);
        assert_eq!(a.join(b).bounds, Some((8, 4, 21, 12)));
        assert_eq!(Area::default().join(a).bounds, a.bounds);
    }

    #[test]
    fn clip_trims_and_drops() {
        let edge = Area::around(0, 0, 5).clip(100, 100);
        assert_eq!(edge.bounds, Some((0, 0, 5, 5)));
        let outside = Area::around(-50, -50, 5).clip(100, 100);
        assert!(outside.empty());
    }
}
