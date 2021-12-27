use crate::{section, utils::Leb128, ErrorKind, ParseError, ParseResult, Vec};
use core::fmt;

pub struct Reader<'a> {
    bytes: &'a [u8],
    idx: usize,
}
pub struct Mark<'a> {
    bytes: &'a [u8],
    idx: usize,
}

impl<'a> Mark<'a> {
    pub fn into_error(self, kind: ErrorKind) -> ParseError<'a> {
        ParseError { mark: self, kind }
    }
    pub fn throw(self, kind: ErrorKind) -> ParseResult<'a> {
        Err(self.into_error(kind))
    }
}

impl fmt::Debug for Mark<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let min = self.idx.saturating_sub(5);
        let max = (self.idx + 5).min(self.bytes.len() - 1);
        writeln!(fmt, "Mark:")?;
        for i in min..=max {
            write!(fmt, "0x{:02X} ", self.bytes[i])?;
        }
        writeln!(fmt)?;
        for _ in min..=0 {
            write!(fmt, "     ")?;
        }
        writeln!(fmt, "^^^^")?;
        Ok(())
    }
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, idx: 0 }
    }

    pub fn mark(&self) -> Mark<'a> {
        Mark {
            bytes: self.bytes,
            idx: self.idx,
        }
    }

    pub fn mark_relative(&self, offset: isize) -> Mark<'a> {
        Mark {
            bytes: self.bytes,
            idx: (self.idx as isize + offset) as usize,
        }
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.idx..]
    }

    pub fn read_str(&mut self) -> ParseResult<'a, &'a str> {
        let mark = self.mark();
        let bytes = self.read_slice()?;
        core::str::from_utf8(bytes)
            .map_err(|inner| mark.into_error(ErrorKind::InvalidUtf8 { inner }))
    }

    pub fn read_slice(&mut self) -> ParseResult<'a, &'a [u8]> {
        let len: usize = self.read_int()?;
        match self.bytes.get(self.idx..self.idx + len) {
            Some(slice) => {
                self.idx += len;
                Ok(slice)
            }
            None => Err(self.mark().into_error(ErrorKind::EndOfFile)),
        }
    }

    pub fn read_vec<FN, T>(&mut self, cb: FN) -> ParseResult<'a, Vec<T>>
    where
        T: 'a,
        FN: for<'b> Fn(&'b mut Reader<'a>) -> ParseResult<'a, T>,
    {
        let len = self.read_int()?;
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(cb(self)?);
        }
        Ok(result)
    }

    pub fn read_until(&mut self, byte: u8) -> &'a [u8] {
        let pos = self
            .bytes
            .iter()
            .skip(self.idx)
            .position(|b| *b == byte)
            .unwrap_or_else(|| {
                panic!(
                    "Could not find byte 0x{:02X}: {:x?}",
                    byte,
                    self.remaining()
                )
            });
        let slice = &self.bytes[self.idx..self.idx + pos];
        self.idx += pos + 1;
        slice
    }

    pub fn read_index<T: section::IndexAlias + Sized>(&mut self) -> ParseResult<'a, T> {
        let val = self.read_int::<u32>()?;
        Ok(T::new(val))
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.len() <= self.idx
    }
    pub fn read_exact<const N: usize>(&mut self) -> ParseResult<'a, [u8; N]> {
        if self.bytes.len() < self.idx + N {
            self.mark().throw(ErrorKind::EndOfFile)?;
        }
        let mut result = [0u8; N];
        result.copy_from_slice(&self.bytes[self.idx..self.idx + N]);
        self.idx += N;
        Ok(result)
    }

    pub fn peek_u8(&mut self) -> ParseResult<'a, u8> {
        self.bytes
            .get(self.idx)
            .copied()
            .ok_or_else(|| self.mark().into_error(ErrorKind::EndOfFile))
    }

    pub fn read_u8(&mut self) -> ParseResult<'a, u8> {
        self.read_exact::<1>().map(|b| b[0])
    }

    pub fn read_u8_if(&mut self, cb: impl FnOnce(u8) -> bool) -> ParseResult<'a, bool> {
        if cb(self.peek_u8()?) {
            self.read_u8()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn read_int<T: Leb128>(&mut self) -> ParseResult<'a, T> {
        T::decode(self)
    }

    pub fn read_f32(&mut self) -> ParseResult<'a, f32> {
        self.read_exact().map(f32::from_le_bytes)
    }

    pub fn read_f64(&mut self) -> ParseResult<'a, f64> {
        self.read_exact().map(f64::from_le_bytes)
    }
}
