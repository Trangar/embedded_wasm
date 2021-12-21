use super::{FuncIdx, GlobalIdx, LabelIdx, LocalIdx, TableIdx, TypeIdx, ValType};
use crate::{Reader, Result, Vec};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Instruction {
    Unreachable, // 0x00
    Nop,         // 0x01
    Block {
        // 0x02
        bt: BlockType,
        inner: Vec<Instruction>,
    },
    Loop {
        // 0x03
        bt: BlockType,
        inner: Vec<Instruction>,
    },
    If {
        // 0x04
        bt: BlockType,
        inner: Vec<Instruction>,
    },
    IfElse {
        // 0x05
        bt: BlockType,
        if_inner: Vec<Instruction>,
        else_inner: Vec<Instruction>,
    },
    Branch {
        // 0x0C
        index: LabelIdx,
    },
    BranchIf {
        // 0x0D
        index: LabelIdx,
    },
    BranchTable {
        // 0x0D
        labels: Vec<LabelIdx>,
        index: LabelIdx,
    },
    Return, // 0x0F
    Call {
        // 0x10
        function: FuncIdx,
    },
    CallIndirect {
        // 0x11
        type_idx: TypeIdx,
        table_idx: TableIdx,
    },
    LocalGet(LocalIdx),   // 0x20
    LocalSet(LocalIdx),   // 0x21
    LocalTee(LocalIdx),   // 0x22
    GlobalGet(GlobalIdx), // 0x23
    GlobalSet(GlobalIdx), // 0x24
    I32Load(MemArg),      // 0x28
    I32Store(MemArg),     // 0x36
    I32Const(i32),        // 0x41
    I64Const(i64),        // 0x42
    I32Add,               // 0x6A
    I32Sub,               // 0x6B
    I32Mul,               // 0x6C
    I32DivUnsigned,       // 0x6E
    I32WrapI64,           // 0xA7
}

impl Instruction {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        match reader.read_u8()? {
            0x03 => Ok(Self::Loop {
                bt: BlockType::parse(reader)?,
                inner: {
                    let slice = reader.read_until(0x0B);
                    reader.read_u8()?;
                    Instruction::parse_vec(&mut Reader::new(slice))?
                },
            }),
            0x0C => Ok(Self::Branch {
                index: reader.read_index()?,
            }),
            0x10 => Ok(Self::Call {
                function: reader.read_index()?,
            }),
            0x11 => Ok(Self::CallIndirect {
                type_idx: reader.read_index()?,
                table_idx: reader.read_index()?,
            }),
            0x20 => Ok(Self::LocalGet(reader.read_index()?)),
            0x21 => Ok(Self::LocalSet(reader.read_index()?)),
            0x22 => Ok(Self::LocalTee(reader.read_index()?)),
            0x23 => Ok(Self::GlobalGet(reader.read_index()?)),
            0x24 => Ok(Self::GlobalSet(reader.read_index()?)),
            0x28 => Ok(Self::I32Load(MemArg::parse(reader)?)),
            0x36 => Ok(Self::I32Store(MemArg::parse(reader)?)),
            0x41 => Ok(Self::I32Const(reader.read_int()?)),
            0x42 => Ok(Self::I64Const(reader.read_int()?)),
            0x6A => Ok(Self::I32Add),
            0x6B => Ok(Self::I32Sub),
            0x6C => Ok(Self::I32Mul),
            0x6E => Ok(Self::I32DivUnsigned),
            0xA7 => Ok(Self::I32WrapI64),
            x => panic!(
                "Unimplemented instruction: 0x{:02X} ({:02X?})",
                x,
                reader.remaining()
            ),
        }
    }
    pub fn parse_vec<'a>(reader: &mut Reader<'a>) -> Result<'a, Vec<Self>> {
        let mut result = Vec::with_capacity(reader.remaining().len());
        while !reader.is_empty() {
            result.push(Self::parse(reader)?);
        }
        result.shrink_to_fit();
        Ok(result)
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum BlockType {
    Empty,
    ValType(ValType),
    Type(TypeIdx),
}

impl BlockType {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        Ok(match reader.read_u8()? {
            0x40 => Self::Empty,
            x => panic!("Unknown blocktype 0x{:02X}", x),
        })
    }
}

#[derive(Clone, Debug)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

impl MemArg {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        let align = reader.read_int()?;
        let offset = reader.read_int()?;
        Ok(Self { align, offset })
    }
}
