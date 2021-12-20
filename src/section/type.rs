use crate::{Error, ErrorKind, Reader, Result};
use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct Type {
    pub lhs: Vec<ValType>,
    pub rhs: Vec<ValType>,
}

impl Type {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        let mark = reader.mark();
        if reader.read_u8()? != 0x60 {
            Err(Error {
                mark,
                kind: ErrorKind::InvalidTypeHeader,
            })
        } else {
            let lhs = reader.read_vec(|r| r.read_val_type())?;
            let rhs = reader.read_vec(|r| r.read_val_type())?;
            Ok(Self { lhs, rhs })
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
    FuncRef = 0x70,
    ExternRef = 0x6F,
}

impl ValType {
    pub fn from_u8(val: u8) -> core::result::Result<Self, ErrorKind> {
        Ok(match val {
            0x7F => Self::I32,
            0x7E => Self::I64,
            0x7D => Self::F32,
            0x7C => Self::F64,
            0x70 => Self::FuncRef,
            0x6F => Self::ExternRef,
            _ => return Err(ErrorKind::UnknownValType),
        })
    }
}
