use std::convert::Infallible;
use std::fmt::Display;
use std::io::Error as IoError;

use thiserror::Error;

use crate::misc::wrapped::RangeError;

/// Error type for protocol and I/O operations.
///
/// This enum unifies all error conditions that may occur while
/// reading, decoding, or validating frames and settings from the
/// high flow NEXT device.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error while reading from a stream or device.
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),

    /// Encountered an invalid or unknown value in the protocol stream.
    ///
    /// Carries the field name and the unexpected numeric value.
    #[error("Invalid or unknown value (name={0}, value={1})")]
    InvalidValue(&'static str, usize),

    /// A decoded value was out of its valid range.
    ///
    /// Wraps a [`RangeError`] describing the bounds violation.
    #[error("Range Error: {0}")]
    RangeError(RangeError<String>),

    /// A CRC checksum mismatch was detected in a frame.
    #[error("Checksum does not match!")]
    ChecksumMismatch,
}

impl<T> From<RangeError<T>> for Error
where
    T: Display,
{
    fn from(value: RangeError<T>) -> Self {
        Self::RangeError(value.to_owned())
    }
}

impl From<Infallible> for Error {
    fn from(_error: Infallible) -> Self {
        unreachable!()
    }
}
