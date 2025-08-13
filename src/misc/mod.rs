//! Miscellaneous utilities for binary I/O, checksums, and typed wrappers.
//!
//! This module collects helper traits and types that are reused across
//! different parts of the codebase.

mod crc;
mod io;
mod wrapped;

pub use self::crc::{CrcReader, CrcWriter};
pub use self::io::{
    Decode, Error as IoError, Guard, GuardOutput, Reader, SkipGuard, SkipReader, ValueGuard,
};
pub use self::wrapped::{RangeError, Ranged, ValueVerifier, Wrapped};
