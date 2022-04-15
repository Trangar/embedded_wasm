mod code;
mod export;
mod import;

pub use self::{code::*, export::*, import::*};

#[cfg(feature = "parse-globals")]
mod global;
#[cfg(feature = "parse-globals")]
pub use self::global::*;

#[cfg(feature = "parse-memory")]
mod memory;
#[cfg(feature = "parse-memory")]
pub use self::memory::*;

#[cfg(feature = "parse-memory")]
use crate::{ParseResult, Reader};
#[cfg(feature = "parse-memory")]
use core::{fmt, num::NonZeroU32};

#[cfg(feature = "parse-memory")]
#[derive(Clone, Debug)]
pub struct Limit {
    pub min: PageSize,
    pub max: Option<PageSize>,
}

#[cfg(feature = "parse-memory")]
impl Limit {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let bit = reader.read_u8()?;
        let min = reader.read_int()?;
        let max = if bit == 0x01 {
            NonZeroU32::new(reader.read_int()?)
        } else {
            None
        };

        Ok(Self {
            min: PageSize(NonZeroU32::new(min).unwrap()),
            max: max.map(PageSize),
        })
    }
}

#[cfg(feature = "parse-memory")]
#[derive(Clone)]
pub struct PageSize(NonZeroU32);

#[cfg(feature = "parse-memory")]
impl fmt::Debug for PageSize {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("PageSize")
            .field("count", &self.0.get())
            .field("bytes", &(self.0.get() * 65536))
            .finish()
    }
}
