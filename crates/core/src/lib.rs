//! # Stepanova Core
//!
//! Platform-independent functionality shared by the device client and the
//! server. Nothing in this crate may depend on device hardware, a display, or
//! a network, so that it builds and tests on any host.

/* CONSTANTS */

/// The version of this crate, as declared in its manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/* TESTS */

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn version_is_declared()
    {
        assert!(!VERSION.is_empty());
    }
}
