mod code;
mod data;
mod export;
mod function;
mod global;
mod import;
mod instruction;
mod memory;
mod r#type;

use core::num::NonZeroU32;

use crate::ErrorKind;
use crate::Reader;
use crate::Result;

pub use self::code::*;
pub use self::data::*;
pub use self::export::*;
pub use self::function::*;
pub use self::global::*;
pub use self::import::*;
pub use self::instruction::*;
pub use self::memory::*;
pub use self::r#type::*;

#[derive(Clone, Debug)]
pub struct Limit {
    pub min: u32,
    pub max: Option<NonZeroU32>,
}

impl Limit {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        let bit = reader.read_u8()?;
        let min = reader.read_int()?;
        let max = if bit == 0x01 {
            NonZeroU32::new(reader.read_int()?)
        } else {
            None
        };

        Ok(Self { min, max })
    }
}

pub trait IndexAlias {
    fn new(val: u32) -> Self;
}

macro_rules! impl_idx {
    ($name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name(pub usize);

        impl IndexAlias for $name {
            fn new(val: u32) -> Self {
                Self(val as usize)
            }
        }
    };
}

impl_idx!(TypeIdx);
impl_idx!(LabelIdx);
impl_idx!(FuncIdx);
impl_idx!(TableIdx);
impl_idx!(MemIdx);
impl_idx!(GlobalIdx);
impl_idx!(LocalIdx);

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
    pub fn from_u8(val: u8) -> core::result::Result<Self, ErrorKind> {
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
