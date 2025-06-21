use std::io::{ErrorKind, Write};

/// A stack allocated, fixed capacity writer.
///
/// `FixedBufferWriter<N>` implements `std::io::Write` and writes to an internal
/// buffer on the stack.
///
/// # Example
///
/// ```rust
/// use std::io::Write;
/// use curium::FixedBufferWriter;
///
/// let mut w: FixedBufferWriter<7> = FixedBufferWriter::new();
/// write!(&mut w, "Hi {}!", "you").unwrap();
/// assert_eq!(w.buffer(), b"Hi you!");
/// ```
pub struct FixedBufferWriter<const COUNT: usize> {
    buffer: [u8; COUNT],
    pos: usize,
}

impl<const COUNT: usize> FixedBufferWriter<COUNT> {
    pub fn new() -> Self {
        Self {
            pos: 0,
            buffer: [0u8; COUNT],
        }
    }

    /// Return a reference to a slice of the underlying buffer.
    pub fn buffer(&self) -> &[u8] {
        &self.buffer[0..self.pos]
    }

    /// Return the actual length of `self.buffer` (number of written bytes)
    pub fn len(&self) -> usize {
        self.pos
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<const COUNT: usize> Default for FixedBufferWriter<COUNT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const COUNT: usize> Write for FixedBufferWriter<COUNT> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let end = self.pos + buf.len();

        if end > COUNT {
            return Err(std::io::Error::from(ErrorKind::WriteZero));
        }

        self.buffer[self.pos..end].copy_from_slice(buf);
        self.pos = end;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::ErrorKind;
    use std::io::Write;

    use super::FixedBufferWriter;

    #[test]
    fn writes_formatted_bytes_into_buffer() {
        let mut w = FixedBufferWriter::<9>::new();
        write!(&mut w, "hello {} {}", core::hint::black_box(1), 1).unwrap();
        let s = String::from_utf8(w.buffer.into()).unwrap();
        assert_eq!(s, "hello 1 1");
    }

    #[test]
    fn write_exact_capacity_succeeds() {
        let mut w = FixedBufferWriter::<5>::new();
        let data = b"abcde";
        let n = w.write(data).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&w.buffer, data);
        assert_eq!(w.pos, 5);
    }

    #[test]
    fn multiple_writes_accumulate() {
        let mut w = FixedBufferWriter::<6>::new();

        w.write_all(b"st").unwrap();
        w.write_all(b"ring").unwrap();
        let s = String::from_utf8(w.buffer.into()).unwrap();
        assert_eq!(s, "string");
        assert_eq!(w.pos, 6);
    }

    #[test]
    fn write_overflow_returns_err() {
        let mut w = FixedBufferWriter::<4>::new();

        w.write_all(b"1234").unwrap();

        // already have 4 bytes '1234', try to write 3 more to overflow
        let err = w.write(b"yyy").unwrap_err();

        assert_eq!(err.kind(), ErrorKind::WriteZero);
        assert_eq!(w.pos, 4);
        assert_eq!(&w.buffer, b"1234");
    }
}
