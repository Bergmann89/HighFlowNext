//! Protocol-level types, encoding and decoding logic.
//!
//! This module defines the wire protocol used by the device and
//! provides encoding and decoding support for binary frames.

pub mod settings;

use crate::misc::{CrcReader, Decode, Guard, GuardOutput, IoError, Reader};

pub use self::settings::Settings;

/// A top-level protocol frame received from or sent to the device.
///
/// Each frame starts with an operation code (`OpCode`), followed by a
/// payload and a trailing CRC checksum. The `Frame` enum provides typed
/// variants for the supported op codes.
///
/// Currently supported:
/// - `0x03` → [`Frame::Settings`]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Frame {
    /// Frame carrying the full device settings (decoded into [`Settings`]).
    Settings(Settings),
}

impl Decode for Frame {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        // Read the operation code
        let op_code = reader.read_u8()?;
        let mut crc = CrcReader::new(reader);

        // Dispatch based on op code
        let ret = match op_code {
            0x03 => {
                let ret = Settings::decode(&mut crc)?;
                R::guard(|x| Self::Settings(x.extract(ret)))
            }
            op_code => Err(IoError::InvalidValue("OpCode", op_code.into()))?,
        };

        // Verify CRC
        let crc_actual = crc.finalize();
        let crc_expceted = reader.read_u16be()?;

        if crc_actual != crc_expceted {
            Err(IoError::ChecksumMismatch)?;
        }

        Ok(ret)
    }
}
