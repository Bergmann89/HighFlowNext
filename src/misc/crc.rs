use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::io::{Result as IoResult, Write};

use crc::{Crc, Digest, Table, CRC_16_USB};

use crate::misc::{IoError, Reader};

/// A writer wrapper that calculates a CRC checksum while writing data.
///
/// The checksum is updated automatically as bytes are written.
/// Once writing is finished, [`finalize`](CrcWriter::finalize) can be used
/// to obtain both the inner writer and the computed CRC value.
pub struct CrcWriter<W> {
    writer: W,
    digest: Digest<'static, u16, Table<1>>,
}

impl<W> CrcWriter<W> {
    /// Creates a new [`CrcWriter`] wrapping the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            digest: CRC.digest(),
        }
    }

    /// Finalizes the CRC calculation and returns the inner writer along with
    /// the computed CRC value.
    pub fn finalize(self) -> (W, u16) {
        let Self { writer, digest } = self;

        let crc = digest.finalize();

        (writer, crc)
    }
}

impl<W> Debug for CrcWriter<W>
where
    W: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("CrcWriter")
            .field("writer", &self.writer)
            .finish_non_exhaustive()
    }
}

impl<W> Write for CrcWriter<W>
where
    W: Write,
{
    /// Writes data to the underlying writer and updates the CRC checksum.
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.digest.update(buf);

        self.writer.write(buf)
    }

    /// Flushes the underlying writer.
    fn flush(&mut self) -> IoResult<()> {
        self.writer.flush()
    }
}

/// A reader wrapper that calculates a CRC checksum while reading data.
///
/// The checksum is updated automatically as bytes are read. After finishing
/// reading, [`finalize`](CrcReader::finalize) returns the computed CRC value.
pub struct CrcReader<'a, R> {
    reader: &'a mut R,
    digest: Digest<'static, u16, Table<1>>,
}

impl<'a, R> CrcReader<'a, R> {
    /// Creates a new [`CrcReader`] wrapping the given reader.
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            reader,
            digest: CRC.digest(),
        }
    }

    /// Finalizes the CRC calculation and returns the computed value.
    #[must_use]
    pub fn finalize(self) -> u16 {
        self.digest.finalize()
    }
}

impl<R> Debug for CrcReader<'_, R>
where
    R: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("CrcReader")
            .field("reader", &self.reader)
            .finish_non_exhaustive()
    }
}

impl<R> Reader for CrcReader<'_, R>
where
    R: Reader,
{
    type Guard = R::Guard;

    /// Reads exactly `buf.len()` bytes from the underlying reader, updating
    /// the CRC checksum with the data.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), IoError> {
        self.reader.read_exact(buf)?;
        self.digest.update(buf);

        Ok(())
    }

    /// Delegates guard creation to the inner reader’s guard.
    fn guard<F, T>(f: F) -> <Self::Guard as super::Guard>::Output<T>
    where
        F: FnOnce(Self::Guard) -> T,
    {
        R::guard(f)
    }
}

/// Constant CRC definition using the USB CRC-16 polynomial.
const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_USB);
