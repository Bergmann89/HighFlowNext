use arrayvec::ArrayVec;

use super::{reader::SkipReader, Error, Guard, GuardOutput, Reader};

/// A trait for decoding values from a binary [`Reader`].
///
/// Implementors define how to parse themselves from a given reader.
/// The trait also provides helpers for optional decoding and skipping
/// over values without fully decoding them.
pub trait Decode: Sized {
    /// Decodes a value of this type from the given reader.
    ///
    /// Returns the decoded value wrapped in the guard of the reader.
    ///
    /// # Errors
    ///
    /// Forwards the error from the reader and raise additional errors if the
    /// decoded type does not have valid values.
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, Error>;

    /// Optionally decodes a value depending on the `read` flag.
    ///
    /// - If `read` is `true`, calls [`Decode::decode`] and wraps the result in `Some`.
    /// - If `read` is `false`, skips the corresponding number of bytes via [`Decode::skip_bytes`]
    ///   and returns `None`.
    ///
    /// # Errors
    ///
    /// See [`Decode::decode`] for more details.
    fn decode_opt<R: Reader>(
        reader: &mut R,
        read: bool,
    ) -> Result<GuardOutput<R, Option<Self>>, Error> {
        if read {
            let val = Self::decode(reader)?;

            Ok(R::guard(|x| Some(x.extract(val))))
        } else {
            Self::skip_bytes(reader)?;

            Ok(R::guard(|_| None))
        }
    }

    /// Skips the bytes corresponding to a value of this type.
    ///
    /// This is implemented by wrapping the provided reader in a [`SkipReader`]
    /// and invoking [`Decode::decode`] to advance its cursor without keeping the value.
    ///
    /// # Errors
    ///
    /// See [`Decode::decode`] for more details.
    fn skip_bytes<R: Reader>(reader: &mut R) -> Result<(), Error> {
        let mut reader = SkipReader(reader);

        Self::decode(&mut reader)?;

        Ok(())
    }
}

impl Decode for u8 {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, Error> {
        let value = reader.read_u8()?;

        Ok(R::guard(|_| value))
    }
}

impl Decode for u16 {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, Error> {
        let value = reader.read_u16be()?;

        Ok(R::guard(|_| value))
    }
}

impl Decode for i16 {
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, Error> {
        let value = reader.read_i16be()?;

        Ok(R::guard(|_| value))
    }
}

impl<T, const N: usize> Decode for [T; N]
where
    T: Decode,
{
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, Error> {
        let mut data = R::guard(|_| ArrayVec::<T, N>::new());

        for _ in 0..N {
            let item = T::decode(reader)?;

            R::guard(|x| {
                let data = x.get_mut(&mut data);
                let item = x.extract(item);

                data.push(item);
            });
        }

        Ok(R::guard(|x| {
            let data = x.extract(data);
            let Ok(data) = data.into_inner() else {
                unreachable!()
            };

            data
        }))
    }
}
