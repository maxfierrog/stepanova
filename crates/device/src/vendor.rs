//! # Vendor Module
//!
//! Unsafe bindings to reMarkable's proprietary e-paper engine
//! (`libqsgepaper.so`, class `EPFramebuffer`), resolved at runtime so that no
//! vendor code is linked or redistributed.
//!
//! The engine exposes no accessor for its framebuffer. It wraps the buffer in
//! a `QImage` while its singleton is constructed, so this module exports the
//! `QImage` pixel constructor, records the arguments of every call made during
//! construction, and forwards each call to Qt's real implementation. The
//! technique follows Quill (<https://github.com/MaximeRivest/quill>, MIT).

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use std::ffi::CStr;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_ulong;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/* CONSTANTS */

const TYPED: &CStr = c"_ZN13EPFramebuffer11swapBuffersE5QRect13EPContentType12EPScreenMode6QFlagsINS_10UpdateFlagEE";
const PLAIN: &CStr = c"_ZN13EPFramebuffer11swapBuffersE5QRect12EPScreenMode6QFlagsINS_10UpdateFlagEE";
const EVENTS: &CStr = c"_ZN16QCoreApplication13processEventsE6QFlagsIN10QEventLoop17ProcessEventsFlagEEi";
const CTOR: &CStr = c"_ZN6QImageC1EPhiixNS_6FormatEPFvPvES2_";
const APP: &CStr = c"_ZN16QCoreApplicationC1ERiPPci";
const INSTANCE: &CStr = c"_ZN13EPFramebuffer8instanceEv";
const LIBRARY: &CStr = c"/usr/lib/plugins/scenegraph/libqsgepaper.so";
const CORE: &CStr = c"libQt6Core.so.6";
const RGB32: c_int = 4;

/* TYPE ALIASES */

type Ctor = unsafe extern "C" fn(
    *mut c_void,
    *mut u8,
    c_int,
    c_int,
    i64,
    c_int,
    *mut c_void,
    *mut c_void,
);
type Typed =
    unsafe extern "C" fn(*mut c_void, Rect, c_int, c_int, c_int) -> c_ulong;
type Plain = unsafe extern "C" fn(*mut c_void, Rect, c_int, c_int) -> c_ulong;
type Events = unsafe extern "C" fn(c_int, c_int);
type App =
    unsafe extern "C" fn(*mut c_void, *mut c_int, *mut *mut c_char, c_int);
type Instance = unsafe extern "C" fn() -> *mut c_void;

/* ENUMERATIONS */

/// The engine's region refresh entry point, whose signature differs between
/// firmware generations.
#[derive(Clone, Copy)]
enum Swap {
    /// OS 5.x (firmware up to 3.28): takes an `EPContentType`.
    Typed(Typed),
    /// OS 6.x (firmware 3.29 onward): no content type.
    Plain(Plain),
}

/* API STRUCTURES */

/// Raw view of the vendor framebuffer, in `QImage::Format_RGB32` layout.
#[derive(Clone, Copy)]
pub(crate) struct Buffer {
    pub(crate) pixels: *mut u8,
    pub(crate) stride: usize,
    pub(crate) height: usize,
    pub(crate) width: usize,
}

/// Handle to the initialized vendor engine.
pub(crate) struct Engine {
    instance: *mut c_void,
    events: Events,
    buffer: Buffer,
    swap: Swap,
}

/// A rectangle in Qt's inclusive-corner `QRect` layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Rect {
    x1: c_int,
    y1: c_int,
    x2: c_int,
    y2: c_int,
}

/* STRUCTURES */

#[derive(Clone, Copy)]
struct Candidate {
    pixels: usize,
    stride: i64,
    height: c_int,
    width: c_int,
    format: c_int,
}

/* IMPLEMENTATIONS */

static CANDIDATES: Mutex<Vec<Candidate>> = Mutex::new(Vec::new());
static REAL: OnceLock<usize> = OnceLock::new();
static CAPTURE: AtomicBool = AtomicBool::new(false);

/// Interposed `QImage::QImage(uchar*, int, int, qsizetype, Format,
/// QImageCleanupFunction, void*)` complete-object constructor.
///
/// # Safety
///
/// Only the dynamic linker should call this, with the arguments of a real
/// `QImage` construction.
#[unsafe(export_name = "_ZN6QImageC1EPhiixNS_6FormatEPFvPvES2_")]
pub unsafe extern "C" fn qimage_ctor(
    this: *mut c_void,
    pixels: *mut u8,
    width: c_int,
    height: c_int,
    stride: i64,
    format: c_int,
    cleanup: *mut c_void,
    info: *mut c_void,
) {
    let real = *REAL.get_or_init(|| unsafe { next(CTOR) });
    let ctor: Ctor = unsafe { std::mem::transmute(real) };
    unsafe {
        ctor(
            this, pixels, width, height, stride, format, cleanup, info,
        )
    };
    if CAPTURE.load(Ordering::Acquire) {
        let pixels = pixels as usize;
        let candidate = Candidate {
            stride,
            height,
            width,
            format,
            pixels,
        };
        if let Ok(mut list) = CANDIDATES.lock() {
            list.push(candidate);
        }
    }
}

impl Engine {
    /// Loads the vendor engine, starts its singleton, and captures its
    /// framebuffer. The engine lives until the process exits.
    pub(crate) fn start() -> Result<Self> {
        unsafe {
            application()?;
            let handle = open(LIBRARY)?;
            let instance: Instance =
                std::mem::transmute(find(handle, INSTANCE)?);
            let swap = resolve(handle)?;
            let core = open(CORE)?;
            let events: Events = std::mem::transmute(find(core, EVENTS)?);
            CAPTURE.store(true, Ordering::Release);
            let instance = instance();
            CAPTURE.store(false, Ordering::Release);
            if instance.is_null() {
                bail!("EPFramebuffer::instance returned null.");
            }
            let buffer = select()?;
            Ok(Self {
                instance,
                events,
                buffer,
                swap,
            })
        }
    }

    /// The captured framebuffer.
    pub(crate) fn buffer(&self) -> Buffer {
        self.buffer
    }

    /// Submits a region for refresh, returning the vendor's update token.
    pub(crate) fn swap(
        &self,
        rect: Rect,
        content: c_int,
        mode: c_int,
        full: bool,
    ) -> c_ulong {
        let flags = full as c_int;
        unsafe {
            match self.swap {
                Swap::Typed(f) => f(self.instance, rect, content, mode, flags),
                Swap::Plain(f) => f(self.instance, rect, mode, flags),
            }
        }
    }

    /// Runs pending Qt events, which the engine's threads may post.
    pub(crate) fn pump(&self) {
        unsafe { (self.events)(0, 0) }
    }
}

impl Rect {
    /// Builds a rectangle from its origin and size.
    pub(crate) fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + w - 1,
            y2: y + h - 1,
        }
    }
}

/* Utilities for Engine */

unsafe fn application() -> Result<()> {
    let core = unsafe { open(CORE)? };
    let app: App = unsafe { std::mem::transmute(find(core, APP)?) };
    let name = Box::leak(Box::new(*b"stepanova\0"));
    let argv = Box::leak(Box::new([
        name.as_mut_ptr().cast::<c_char>(),
        std::ptr::null_mut(),
    ]));
    let argc = Box::leak(Box::new(1 as c_int));
    let object = Box::leak(Box::new([0u64; 16]));
    unsafe {
        app(
            object.as_mut_ptr().cast(),
            argc,
            argv.as_mut_ptr(),
            0x060800,
        )
    };
    Ok(())
}

unsafe fn resolve(handle: *mut c_void) -> Result<Swap> {
    if let Ok(f) = unsafe { find(handle, TYPED) } {
        return Ok(Swap::Typed(unsafe {
            std::mem::transmute::<*mut c_void, Typed>(f)
        }));
    }
    let f = unsafe { find(handle, PLAIN) }
        .context("No known EPFramebuffer::swapBuffers signature.")?;
    Ok(Swap::Plain(unsafe {
        std::mem::transmute::<*mut c_void, Plain>(f)
    }))
}

unsafe fn open(name: &CStr) -> Result<*mut c_void> {
    let flags = libc::RTLD_NOW | libc::RTLD_GLOBAL;
    let handle = unsafe { libc::dlopen(name.as_ptr(), flags) };
    if handle.is_null() {
        bail!(
            "Failed to load {}: {}.",
            name.to_string_lossy(),
            unsafe { error() }
        );
    }
    Ok(handle)
}

unsafe fn find(handle: *mut c_void, name: &CStr) -> Result<*mut c_void> {
    let symbol = unsafe { libc::dlsym(handle, name.as_ptr()) };
    if symbol.is_null() {
        bail!("Missing symbol {}.", name.to_string_lossy());
    }
    Ok(symbol)
}

unsafe fn next(name: &CStr) -> usize {
    let symbol = unsafe { libc::dlsym(libc::RTLD_NEXT, name.as_ptr()) };
    if symbol.is_null() {
        std::process::abort();
    }
    symbol as usize
}

unsafe fn error() -> String {
    let text = unsafe { libc::dlerror() };
    if text.is_null() {
        return "unknown error".into();
    }
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

fn select() -> Result<Buffer> {
    let list = CANDIDATES
        .lock()
        .ok()
        .context("Framebuffer candidate list was poisoned.")?;
    let mut valid: Vec<Candidate> = list
        .iter()
        .copied()
        .filter(plausible)
        .collect();
    valid.sort_by_key(|c| c.pixels);
    valid.dedup_by_key(|c| c.pixels);
    match valid.as_slice() {
        [one] => Ok(Buffer {
            pixels: one.pixels as *mut u8,
            stride: one.stride as usize,
            height: one.height as usize,
            width: one.width as usize,
        }),
        [] => bail!("No framebuffer was observed during engine startup."),
        _ => bail!(
            "{} plausible framebuffers were observed.",
            valid.len()
        ),
    }
}

fn plausible(c: &Candidate) -> bool {
    c.pixels != 0
        && c.width > 0
        && c.height > 0
        && c.format == RGB32
        && c.stride >= i64::from(c.width) * 4
}
