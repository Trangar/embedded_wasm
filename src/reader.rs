use crate::{section, utils::Leb128, Error, ErrorKind, Result, Vec};
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
    pub fn to_error(self, kind: ErrorKind) -> Error<'a> {
        Error { mark: self, kind }
    }
    pub fn throw(self, kind: ErrorKind) -> Result<'a> {
        Err(self.to_error(kind))
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

    pub fn read_str(&mut self) -> Result<'a, &'a str> {
        let mark = self.mark();
        let bytes = self.read_slice()?;
        core::str::from_utf8(bytes).map_err(|inner| mark.to_error(ErrorKind::InvalidUtf8 { inner }))
    }

    pub fn read_slice(&mut self) -> Result<'a, &'a [u8]> {
        let len: usize = self.read_int()?;
        match self.bytes.get(self.idx..self.idx + len) {
            Some(slice) => {
                self.idx += len;
                Ok(slice)
            }
            None => Err(self.mark().to_error(ErrorKind::EndOfFile)),
        }
    }

    pub fn read_vec<FN, T>(&mut self, cb: FN) -> Result<'a, Vec<T>>
    where
        T: 'a,
        FN: for<'b> Fn(&'b mut Reader<'a>) -> Result<'a, T>,
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
        self.idx += pos;
        slice
    }

    pub fn read_index<T: section::IndexAlias + Sized>(&mut self) -> Result<'a, T> {
        let val = self.read_int::<u32>()?;
        Ok(T::new(val))
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.len() <= self.idx
    }
    pub fn read_exact<const N: usize>(&mut self) -> Result<'a, [u8; N]> {
        if self.bytes.len() < self.idx + N {
            self.mark().throw(ErrorKind::EndOfFile)?;
        }
        let mut result = [0u8; N];
        result.copy_from_slice(&self.bytes[self.idx..self.idx + N]);
        self.idx += N;
        Ok(result)
    }

    pub fn read_u8(&mut self) -> Result<'a, u8> {
        self.read_exact::<1>().map(|b| b[0])
    }

    fn read_and_map_u8<F, T>(&mut self, f: F) -> Result<'a, T>
    where
        F: FnOnce(u8) -> core::result::Result<T, ErrorKind>,
    {
        let mark = self.mark();
        let val = self.read_u8()?;
        f(val).map_err(|kind| mark.to_error(kind))
    }

    pub fn read_section_type(&mut self) -> Result<'a, section::SectionType> {
        self.read_and_map_u8(section::SectionType::from_u8)
    }

    pub fn read_val_type(&mut self) -> Result<'a, section::ValType> {
        self.read_and_map_u8(section::ValType::from_u8)
    }

    pub fn read_int<T: Leb128>(&mut self) -> Result<'a, T> {
        T::decode(self)
    }

    #[deprecated(note = "Use self.read_int::<usize>() instead")]
    pub fn read_len(&mut self) -> Result<'a, usize> {
        self.read_int()
    }
}
