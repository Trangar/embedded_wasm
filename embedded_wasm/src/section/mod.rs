mod code;
// mod data;
mod export;
// mod function;
// mod global;
mod import;
// mod memory;
// mod table;
mod r#type;

use crate::{ErrorKind, ParseResult, Reader};
use core::fmt;
// , num::NonZeroU32

pub use self::{code::*, export::*, import::*, r#type::*};

// #[derive(Clone, Debug)]
// pub struct Limit {
//     pub min: u32,
//     pub max: Option<NonZeroU32>,
// }

// impl Limit {
//     pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
//         let bit = reader.read_u8()?;
//         let min = reader.read_int()?;
//         let max = if bit == 0x01 {
//             NonZeroU32::new(reader.read_int()?)
//         } else {
//             None
//         };

//         Ok(Self { min, max })
//     }
// }

pub trait IndexAlias {
    fn new(val: u32) -> Self;
}

macro_rules! impl_idx {
    ($name:ident (prefix: $prefix:expr)) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub usize);

        impl fmt::Debug for $name {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "{}{}", $prefix, self.0)
            }
        }

        impl IndexAlias for $name {
            fn new(val: u32) -> Self {
                Self(val as usize)
            }
        }
    };
}

impl_idx!(TypeIdx (prefix: "$t"));
impl_idx!(LabelIdx (prefix: "$L"));
impl_idx!(FuncIdx (prefix: "$f"));
impl_idx!(TableIdx (prefix: "$t"));
impl_idx!(MemIdx (prefix: "$m"));
impl_idx!(GlobalIdx (prefix: "$g"));
impl_idx!(LocalIdx (prefix: "$l"));
impl_idx!(ElemIdx (prefix: "$e"));
impl_idx!(DataIdx (prefix: "$d"));

#[derive(Debug)]
pub enum SectionType {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
    DataCount = 12,
}

impl SectionType {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        let val = reader.read_u8()?;
        Self::from_u8(val).map_err(|kind| mark.into_error(kind))
    }
    pub fn from_u8(val: u8) -> Result<Self, ErrorKind> {
        Ok(match val {
            0 => Self::Custom,
            1 => Self::Type,
            2 => Self::Import,
            3 => Self::Function,
            4 => Self::Table,
            5 => Self::Memory,
            6 => Self::Global,
            7 => Self::Export,
            8 => Self::Start,
            9 => Self::Element,
            10 => Self::Code,
            11 => Self::Data,
            12 => Self::DataCount,
            _ => return Err(ErrorKind::InvalidSection),
        })
    }
}
