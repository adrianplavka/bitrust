use crate::error::{Error, Result};

/// Trait used by the deserializer for iterating over input. This is manually
/// "specialized" for iterating over &[u8].
///
/// This trait is sealed and cannot be implemented for types outside of this
/// crate.
pub trait Read<'de>: private::Sealed {
    /// Peek at the current byte in the input, without consuming it.
    #[doc(hidden)]
    fn peek_byte(&self) -> Result<u8>;

    /// Peek at the n-th byte in the input from the current index,
    /// without consuming it.
    #[doc(hidden)]
    fn peek_byte_nth(&self, n: usize) -> Result<u8>;

    /// Consumes the next byte in the input.
    #[doc(hidden)]
    fn next_byte(&mut self) -> Result<u8>;

    /// Consumes next bytes in the input until the length of inclusive end.
    #[doc(hidden)]
    fn next_bytes(&mut self, end: usize) -> Result<&'de [u8]>;

    // Check, if input is at end.
    #[doc(hidden)]
    fn end(&self) -> bool;
}

/// Bencode input source that reads from a slice of bytes.
pub struct SliceRead<'a> {
    pub slice: &'a [u8],

    /// Index of the current byte, used by `peek_*` & `next_*` methods.
    index: usize,
}

/// Bencode input source that reads from an UTF-8 string.
pub struct StrRead<'a> {
    delegate: SliceRead<'a>,
}

// Prevent users from implementing the Read trait.
mod private {
    pub trait Sealed {}
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> SliceRead<'a> {
    /// Creates a Bencode input source to read from a slice of bytes.
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead {
            slice: slice,
            index: 0,
        }
    }
}

impl<'a> private::Sealed for SliceRead<'a> {}

impl<'a> Read<'a> for SliceRead<'a> {
    fn peek_byte(&self) -> Result<u8> {
        if self.index < self.slice.len() {
            Ok(self.slice[self.index])
        } else {
            Err(Error::EOF)
        }
    }

    fn peek_byte_nth(&self, n: usize) -> Result<u8> {
        if self.index + n < self.slice.len() {
            Ok(self.slice[self.index + n])
        } else {
            Err(Error::EOF)
        }
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.index += 1;
        Ok(byte)
    }

    fn next_bytes(&mut self, end: usize) -> Result<&'a [u8]> {
        if self.index + end < self.slice.len() {
            let bytes = &self.slice[self.index..=self.index + end];
            self.index += end + 1;
            Ok(bytes)
        } else {
            Err(Error::EOF)
        }
    }

    fn end(&self) -> bool {
        self.index == self.slice.len()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> StrRead<'a> {
    /// Creates a Bencode input source to read from an UTF-8 string.
    pub fn new(s: &'a str) -> Self {
        StrRead {
            delegate: SliceRead::new(s.as_bytes()),
        }
    }
}

impl<'a> private::Sealed for StrRead<'a> {}

impl<'a> Read<'a> for StrRead<'a> {
    fn peek_byte(&self) -> Result<u8> {
        self.delegate.peek_byte()
    }

    fn peek_byte_nth(&self, n: usize) -> Result<u8> {
        self.delegate.peek_byte_nth(n)
    }

    fn next_byte(&mut self) -> Result<u8> {
        self.delegate.next_byte()
    }

    fn next_bytes(&mut self, end: usize) -> Result<&'a [u8]> {
        self.delegate.next_bytes(end)
    }

    fn end(&self) -> bool {
        self.delegate.end()
    }
}
