use crate::{Error, ErrorKind, Reader, Result, Vec};

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
    Num(NumType),
    Ref(RefType),
}

impl ValType {
    pub fn from_u8(val: u8) -> core::result::Result<Self, ErrorKind> {
        Ok(match val {
            0x7F => Self::Num(NumType::I32),
            0x7E => Self::Num(NumType::I64),
            0x7D => Self::Num(NumType::F32),
            0x7C => Self::Num(NumType::F64),
            0x70 => Self::Ref(RefType::FuncRef),
            0x6F => Self::Ref(RefType::ExternRef),
            _ => return Err(ErrorKind::UnknownValType),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefType {
    FuncRef = 0x70,
    ExternRef = 0x6F,
}

impl RefType {
    pub fn from_u8(val: u8) -> core::result::Result<Self, ErrorKind> {
        Ok(match val {
            0x70 => Self::FuncRef,
            0x6F => Self::ExternRef,
            _ => return Err(ErrorKind::UnknownRefType),
        })
    }
}
