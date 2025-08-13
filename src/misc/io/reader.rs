use std::io::Read;
use std::marker::PhantomData;

use super::Error;

/// A trait representing a binary reader abstraction used for decoding values.
///
/// Implementations provide methods to read primitive values (`u8`, `u16`, etc.)
/// and to manage a [`Guard`] type that wraps decoded results.
///
/// The `Guard` mechanism allows the same decode logic to be reused in both
/// *normal* mode (returning real values) and *skip* mode (advancing the cursor
/// without keeping values).
pub trait Reader: Sized {
    /// The [`Guard`] implementation associated with this reader.
    type Guard: Guard;

    /// Reads exactly the number of bytes into the provided buffer.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    /// Skips over `N` bytes in the input.
    fn skip<const N: usize>(&mut self) -> Result<(), Error> {
        self.read_exact(&mut [0; N])?;

        Ok(())
    }

    /// Reads an unsigned 8-bit integer.
    fn read_u8(&mut self) -> Result<u8, Error> {
        let mut buf = [0; 1];

        self.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    /// Reads a big-endian unsigned 16-bit integer.
    fn read_u16be(&mut self) -> Result<u16, Error> {
        let mut buf = [0; 2];

        self.read_exact(&mut buf)?;

        Ok(u16::from_be_bytes(buf))
    }

    /// Reads a big-endian signed 16-bit integer.
    fn read_i16be(&mut self) -> Result<i16, Error> {
        let mut buf = [0; 2];

        self.read_exact(&mut buf)?;

        Ok(i16::from_be_bytes(buf))
    }

    /// Creates a guarded output by invoking the given closure with the [`Guard`] instance.
    fn guard<F, T>(f: F) -> <Self::Guard as Guard>::Output<T>
    where
        F: FnOnce(Self::Guard) -> T;
}

/// Helper alias for the output of a [`Reader`] using its [`Guard`].
pub type GuardOutput<R, T> = <<R as Reader>::Guard as Guard>::Output<T>;

/// Defines how to wrap, access, and extract values during decoding.
///
/// - In **normal mode** (see [`ValueGuard`]), `Output<T>` is just `T`.
/// - In **skip mode** (see [`SkipGuard`]), `Output<T>` is `PhantomData<T>` to avoid allocation.
pub trait Guard {
    /// Type of wrapped output for a value of type `T`.
    type Output<T>;

    /// Accesses a reference to the contained value.
    fn get<'a, T>(&self, value: &'a Self::Output<T>) -> &'a T;

    /// Accesses a mutable reference to the contained value.
    fn get_mut<'a, T>(&self, value: &'a mut Self::Output<T>) -> &'a mut T;

    /// Consumes the wrapped output and extracts the inner value.
    fn extract<T>(&self, value: Self::Output<T>) -> T;

    /// Transposes a `Result` inside the guard’s output into a `Result` of wrapped output.
    fn transpose_result<T, E>(res: Self::Output<Result<T, E>>) -> Result<Self::Output<T>, E>;
}

impl<X> Reader for X
where
    X: Read,
{
    type Guard = ValueGuard;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        Ok(Read::read_exact(self, buf)?)
    }

    fn guard<F, T>(f: F) -> <Self::Guard as Guard>::Output<T>
    where
        F: FnOnce(Self::Guard) -> T,
    {
        f(ValueGuard)
    }
}

/// A wrapper around a [`Reader`] that discards values instead of keeping them.
///
/// Used by [`Decode::skip_bytes`](super::Decode::skip_bytes).
#[derive(Debug)]
pub struct SkipReader<'a, R: Reader>(pub(super) &'a mut R);

impl<R> Reader for SkipReader<'_, R>
where
    R: Reader,
{
    type Guard = SkipGuard;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.0.read_exact(buf)
    }

    fn guard<F, T>(f: F) -> <Self::Guard as Guard>::Output<T>
    where
        F: FnOnce(Self::Guard) -> T,
    {
        let _ = f;
        PhantomData
    }
}

/// A [`Guard`] implementation for **normal decoding mode**.
/// Values are preserved and accessible.
#[derive(Debug)]
pub struct ValueGuard;

impl Guard for ValueGuard {
    type Output<T> = T;

    #[inline]
    fn transpose_result<T, E>(res: Self::Output<Result<T, E>>) -> Result<Self::Output<T>, E> {
        res
    }

    #[inline]
    fn get<'a, T>(&self, value: &'a Self::Output<T>) -> &'a T {
        value
    }

    #[inline]
    fn get_mut<'a, T>(&self, value: &'a mut Self::Output<T>) -> &'a mut T {
        value
    }

    #[inline]
    fn extract<T>(&self, value: Self::Output<T>) -> T {
        value
    }
}

/// A [`Guard`] implementation for **skip mode**.
/// Values are replaced with [`PhantomData`] and cannot be accessed.
#[derive(Debug)]
pub struct SkipGuard;

impl Guard for SkipGuard {
    type Output<T> = PhantomData<T>;

    #[inline]
    fn transpose_result<T, E>(res: Self::Output<Result<T, E>>) -> Result<Self::Output<T>, E> {
        let _res = res;
        Ok(PhantomData)
    }

    #[inline]
    fn get<'a, T>(&self, _value: &'a Self::Output<T>) -> &'a T {
        panic!("Unable to get value in `Skip` mode!");
    }

    #[inline]
    fn get_mut<'a, T>(&self, _value: &'a mut Self::Output<T>) -> &'a mut T {
        panic!("Unable to get value in `Skip` mode!");
    }

    #[inline]
    fn extract<T>(&self, _value: Self::Output<T>) -> T {
        panic!("Unable to extract value in `Skip` mode!");
    }
}
