//! # Input Module
//!
//! Reads the Paper Pro's pen, touchscreen, and power button straight from
//! their Linux event devices, one reader thread per device, and delivers
//! decoded [`Event`]s over a channel.

use anyhow::Context;
use anyhow::Result;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

/* CONSTANTS */

const ABS_MT_TRACKING_ID: u16 = 57;
const BTN_TOOL_RUBBER: u16 = 321;
const ABS_PRESSURE: u16 = 24;
const ABS_MT_SLOT: u16 = 47;
const BTN_TOUCH: u16 = 330;
const KEY_POWER: u16 = 116;
const ABS_X: u16 = 0;
const ABS_Y: u16 = 1;
const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const EV_ABS: u16 = 3;

const EVIOCGNAME: u64 = (2 << 30) | ((NAME as u64) << 16) | (0x45 << 8) | 0x06;
const PEN_MAX_X: i64 = 11180;
const PEN_MAX_Y: i64 = 15340;
const ABSINFO: u64 = 24;
const SLOTS: usize = 16;
const NAME: usize = 256;
const SIZE: usize = 24;

/* ENUMERATIONS */

/// A decoded input event.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
    Pen(Sample),
    Fingers(usize),
    Power,
}

/// Which physical device a reader serves.
#[derive(Clone, Copy, Debug)]
enum Source {
    Power,
    Touch,
    Pen,
}

/* API STRUCTURES */

/// One complete pen report, in screen pixels.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sample {
    pub(crate) time: SystemTime,
    pub(crate) pressure: i32,
    pub(crate) touching: bool,
    pub(crate) eraser: bool,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

/* STRUCTURES */

#[derive(Clone, Copy)]
struct Raw {
    time: SystemTime,
    value: i32,
    kind: u16,
    code: u16,
}

#[derive(Default)]
struct Pen {
    range: (i64, i64),
    pressure: i32,
    touching: bool,
    eraser: bool,
    x: i64,
    y: i64,
}

#[derive(Default)]
struct Touch {
    active: [bool; SLOTS],
    slot: usize,
}

/* IMPLEMENTATIONS */

/// Starts readers for every input device, scaling pen coordinates to a
/// `width` by `height` screen.
pub(crate) fn listen(width: i32, height: i32) -> Result<Receiver<Event>> {
    let (tx, rx) = channel();
    [Source::Pen, Source::Touch, Source::Power]
        .into_iter()
        .try_for_each(|source| spawn(source, tx.clone(), width, height))?;
    Ok(rx)
}

impl Source {
    fn marker(self) -> &'static str {
        match self {
            Source::Power => "powerkey",
            Source::Touch => "touch",
            Source::Pen => "marker",
        }
    }
}

impl Pen {
    fn apply(&mut self, raw: Raw) {
        match (raw.kind, raw.code) {
            (EV_ABS, ABS_PRESSURE) => self.pressure = raw.value,
            (EV_KEY, BTN_TOOL_RUBBER) => self.eraser = raw.value != 0,
            (EV_KEY, BTN_TOUCH) => self.touching = raw.value != 0,
            (EV_ABS, ABS_X) => self.x = raw.value.into(),
            (EV_ABS, ABS_Y) => self.y = raw.value.into(),
            _ => {},
        }
    }

    fn sample(&self, time: SystemTime, width: i32, height: i32) -> Sample {
        let x = self.x * i64::from(width - 1) / self.range.0.max(1);
        let y = self.y * i64::from(height - 1) / self.range.1.max(1);
        Sample {
            pressure: self.pressure,
            touching: self.touching,
            eraser: self.eraser,
            x: x as i32,
            y: y as i32,
            time,
        }
    }
}

impl Pen {
    fn new(file: &File) -> Self {
        let x = maximum(file, ABS_X).unwrap_or(PEN_MAX_X);
        let y = maximum(file, ABS_Y).unwrap_or(PEN_MAX_Y);
        Self {
            range: (x, y),
            ..Self::default()
        }
    }
}

impl Touch {
    fn apply(&mut self, raw: Raw) -> Option<usize> {
        match (raw.kind, raw.code) {
            (EV_ABS, ABS_MT_SLOT) => {
                self.slot = usize::try_from(raw.value)
                    .unwrap_or(0)
                    .min(SLOTS - 1);
                None
            },
            (EV_ABS, ABS_MT_TRACKING_ID) => {
                self.active[self.slot] = raw.value != -1;
                Some(
                    self.active
                        .iter()
                        .filter(|a| **a)
                        .count(),
                )
            },
            _ => None,
        }
    }
}

/* Utilities for listen */

fn spawn(
    source: Source,
    tx: Sender<Event>,
    width: i32,
    height: i32,
) -> Result<()> {
    let file = locate(source.marker()).with_context(|| {
        format!("No input device named *{}*.", source.marker())
    })?;
    thread::spawn(move || read(source, file, tx, width, height));
    Ok(())
}

fn read(source: Source, mut file: File, tx: Sender<Event>, w: i32, h: i32) {
    let mut pen = Pen::new(&file);
    let mut touch = Touch::default();
    let mut bytes = [0u8; SIZE * 64];
    while let Ok(n) = file.read(&mut bytes) {
        let events = bytes[..n]
            .as_chunks::<SIZE>()
            .0
            .iter()
            .map(decode)
            .filter_map(|raw| match source {
                Source::Pen => pen_event(&mut pen, raw, w, h),
                Source::Touch => touch
                    .apply(raw)
                    .map(Event::Fingers),
                Source::Power => power_event(raw),
            });
        if events
            .map(|e| tx.send(e))
            .any(|r| r.is_err())
        {
            return;
        }
    }
}

fn pen_event(pen: &mut Pen, raw: Raw, w: i32, h: i32) -> Option<Event> {
    if raw.kind == EV_SYN {
        return Some(Event::Pen(pen.sample(raw.time, w, h)));
    }
    pen.apply(raw);
    None
}

fn power_event(raw: Raw) -> Option<Event> {
    (raw.kind == EV_KEY && raw.code == KEY_POWER && raw.value == 1)
        .then_some(Event::Power)
}

fn decode(chunk: &[u8; SIZE]) -> Raw {
    let word =
        |i: usize| i64::from_le_bytes(chunk[i..i + 8].try_into().unwrap());
    let half = |i: usize| u16::from_le_bytes([chunk[i], chunk[i + 1]]);
    let secs = Duration::from_secs(word(0).max(0) as u64);
    let micros = Duration::from_micros(word(8).max(0) as u64);
    Raw {
        value: i32::from_le_bytes(chunk[20..24].try_into().unwrap()),
        time: SystemTime::UNIX_EPOCH + secs + micros,
        kind: half(16),
        code: half(18),
    }
}

fn locate(marker: &str) -> Option<File> {
    let mut nodes: Vec<PathBuf> = fs::read_dir("/dev/input")
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("event"))
        })
        .collect();
    nodes.sort();
    nodes
        .iter()
        .filter_map(|path| File::open(path).ok())
        .find(|file| {
            name(file).is_some_and(|n| n.to_lowercase().contains(marker))
        })
}

fn maximum(file: &File, axis: u16) -> Option<i64> {
    let request =
        (2 << 30) | (ABSINFO << 16) | (0x45 << 8) | (0x40 + u64::from(axis));
    let mut info = [0i32; 6];
    let fd = file.as_raw_fd();
    let status = unsafe { libc::ioctl(fd, request as _, info.as_mut_ptr()) };
    (status >= 0 && info[2] > 0).then_some(i64::from(info[2]))
}

fn name(file: &File) -> Option<String> {
    let mut buffer = [0u8; NAME];
    let fd = file.as_raw_fd();
    let len = unsafe { libc::ioctl(fd, EVIOCGNAME as _, buffer.as_mut_ptr()) };
    let len = usize::try_from(len)
        .ok()?
        .min(NAME);
    let text = buffer[..len]
        .split(|b| *b == 0)
        .next()?;
    Some(String::from_utf8_lossy(text).into_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    fn event(kind: u16, code: u16, value: i32) -> [u8; SIZE] {
        let mut bytes = [0u8; SIZE];
        bytes[0..8].copy_from_slice(&5i64.to_le_bytes());
        bytes[16..18].copy_from_slice(&kind.to_le_bytes());
        bytes[18..20].copy_from_slice(&code.to_le_bytes());
        bytes[20..24].copy_from_slice(&value.to_le_bytes());
        bytes
    }

    #[test]
    fn decodes_and_scales_pen_report() {
        let mut pen = Pen {
            range: (PEN_MAX_X, PEN_MAX_Y),
            ..Pen::default()
        };
        let frames = [
            event(EV_ABS, ABS_X, 11180),
            event(EV_ABS, ABS_Y, 0),
            event(EV_ABS, ABS_PRESSURE, 2048),
            event(EV_KEY, BTN_TOUCH, 1),
            event(EV_SYN, 0, 0),
        ];
        let out: Vec<Event> = frames
            .iter()
            .map(decode)
            .filter_map(|raw| pen_event(&mut pen, raw, 1620, 2160))
            .collect();
        let [Event::Pen(sample)] = out.as_slice() else {
            panic!("expected one pen sample, got {out:?}");
        };
        assert_eq!((sample.x, sample.y), (1619, 0));
        assert!(sample.touching && !sample.eraser);
        assert_eq!(sample.pressure, 2048);
    }

    #[test]
    fn counts_fingers() {
        let mut touch = Touch::default();
        let count = (0..5)
            .flat_map(|s| {
                [
                    event(EV_ABS, ABS_MT_SLOT, s),
                    event(EV_ABS, ABS_MT_TRACKING_ID, s + 100),
                ]
            })
            .filter_map(|f| touch.apply(decode(&f)))
            .last();
        assert_eq!(count, Some(5));
    }
}
