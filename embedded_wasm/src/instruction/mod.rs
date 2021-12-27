mod execute;
mod parse;

pub use self::{execute::*, parse::*};

use crate::{
    section::{
        DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, RefType, TableIdx, TypeIdx,
        ValType,
    },
    ParseResult, Reader, Vec,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    // 5.4.1 Control instructions
    Unreachable,
    Nop,
    Block {
        bt: BlockType,
        inner: Vec<Instruction>,
    },
    Loop {
        bt: BlockType,
        inner: Vec<Instruction>,
    },
    If {
        bt: BlockType,
        inner: Vec<Instruction>,
    },
    IfElse {
        bt: BlockType,
        if_inner: Vec<Instruction>,
        else_inner: Vec<Instruction>,
    },
    Branch {
        index: LabelIdx,
    },
    BranchIf {
        index: LabelIdx,
    },
    BranchTable {
        labels: Vec<LabelIdx>,
        index: LabelIdx,
    },
    Return,
    Call {
        function: FuncIdx,
    },
    CallIndirect {
        type_idx: TypeIdx,
        table_idx: TableIdx,
    },

    // 5.4.2 Reference instructions
    RefNull {
        reftype: RefType,
    },
    RefIsNull,
    RefFunc {
        index: FuncIdx,
    },

    // 5.4.3 Parametric instructions
    Drop,
    Select,
    SelectVal {
        val: Vec<ValType>,
    },

    // 5.4.4 Variable instructions
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    LocalTee(LocalIdx),
    GlobalGet(GlobalIdx),
    GlobalSet(GlobalIdx),

    // 5.4.5 Table instructions
    TableGet {
        index: TableIdx,
    },
    TableSet {
        index: TableIdx,
    },
    TableInit {
        y: ElemIdx,
        x: TableIdx,
    },
    TableDrop {
        x: ElemIdx,
    },
    TableCopy {
        x: TableIdx,
        y: TableIdx,
    },
    TableGrow {
        x: TableIdx,
    },
    TableSize {
        x: TableIdx,
    },
    TableFill {
        x: TableIdx,
    },

    // 5.4.6 Memory instructions
    I32Load(MemArg),
    I64Load(MemArg),
    F32Load(MemArg),
    F64Load(MemArg),
    I32Load8S(MemArg),
    I32Load8U(MemArg),
    I32Load16S(MemArg),
    I32Load16U(MemArg),
    I64Load8S(MemArg),
    I64Load8U(MemArg),
    I64Load16S(MemArg),
    I64Load16U(MemArg),
    I64Load32S(MemArg),
    I64Load32U(MemArg),
    I32Store(MemArg),
    I64Store(MemArg),
    F32Store(MemArg),
    F64Store(MemArg),
    I32Store8(MemArg),
    I32Store16(MemArg),
    I64Store8(MemArg),
    I64Store16(MemArg),
    I64Store32(MemArg),
    MemorySize,
    MemoryGrow,
    MemoryInit {
        index: DataIdx,
    },
    DataDrop {
        index: DataIdx,
    },
    MemoryCopy,
    MemoryFill,

    // Numeric instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    I32EqualZero,
    I32Equals,
    I32NotEquals,
    I32LessThanSigned,
    I32LessThanUnsigned,
    I32GreaterThanSigned,
    I32GreaterThanUnsigned,
    I32LessOrEqualToSigned,
    I32LessOrEqualToUnsigned,
    I32GreaterOrEqualToSigned,
    I32GreaterOrEqualToUnsigned,

    I64EqualZero,
    I64Equals,
    I64NotEquals,
    I64LessThanSigned,
    I64LessThanUnsigned,
    I64GreaterThanSigned,
    I64GreaterThanUnsigned,
    I64LessOrEqualToSigned,
    I64LessOrEqualToUnsigned,
    I64GreaterOrEqualToSigned,
    I64GreaterOrEqualToUnsigned,

    F32Equals,
    F32NotEquals,
    F32LessThan,
    F32GreaterThan,
    F32LessOrEqualTo,
    F32GreaterOrEqualTo,

    F64Equals,
    F64NotEquals,
    F64LessThan,
    F64GreaterThan,
    F64LessOrEqualTo,
    F64GreaterOrEqualTo,

    I32CountLeadingZeroBits,
    I32CountTrailingZeroBits,
    I32CountNonZeroBits,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivSigned,
    I32DivUnsigned,
    I32RemainderSigned,
    I32RemainderUnsigned,
    I32LogicalAnd,
    I32LogicalOr,
    I32LogicalXor,
    I32ShiftLeft,
    I32ShiftRightSigned,
    I32ShiftRightUnsigned,
    I32RotateLeft,
    I32RotateRight,

    I64CountLeadingZeroBits,
    I64CountTrailingZeroBits,
    I64CountNonZeroBits,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivSigned,
    I64DivUnsigned,
    I64RemainderSigned,
    I64RemainderUnsigned,
    I64LogicalAnd,
    I64LogicalOr,
    I64LogicalXor,
    I64ShiftLeft,
    I64ShiftRightSigned,
    I64ShiftRightUnsigned,
    I64RotateLeft,
    I64RotateRight,

    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32CopySign,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64CopySign,

    I32WrapI64,
    I32TruncF32Signed,
    I32TruncF32Unsigned,
    I32TruncF64Signed,
    I32TruncF64Unsigned,

    I64ExtendI32Signed,
    I64ExtendI32Unsigned,
    I64TruncF32Signed,
    I64TruncF32Unsigned,
    I64TruncF64Signed,
    I64TruncF64Unsigned,

    F32ConvertI32Signed,
    F32ConvertI32Unsigned,
    F32ConvertI64Signed,
    F32ConvertI64Unsigned,
    F32DemoteF64,

    F64ConvertI32Signed,
    F64ConvertI32Unsigned,
    F64ConvertI64Signed,
    F64ConvertI64Unsigned,
    F64PromoteF32,

    I32ReinterpretAsF32,
    I64ReinterpretAsF64,
    F32ReinterpretAsI32,
    F64ReinterpretAsI64,

    I32Extend8Signed,
    I32Extend16Signed,
    I64Extend8Signed,
    I64Extend16Signed,
    I64Extend32Signed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    ValType(ValType),
    Type(TypeIdx),
}

impl BlockType {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Ok(match reader.read_u8()? {
            0x40 => Self::Empty,
            x => panic!("Unknown blocktype 0x{:02X}", x),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

impl MemArg {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let align = reader.read_int()?;
        let offset = reader.read_int()?;
        Ok(Self { align, offset })
    }
}
