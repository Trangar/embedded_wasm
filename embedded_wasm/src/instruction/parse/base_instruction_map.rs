use super::{MemArg, NumType, ValType};
use crate::{
    instruction::{BlockType, Instruction, RefType, Signedness},
    ErrorKind, Mark, ParseResult, Reader,
};

fn unknown_instruction<'a>(_: &mut Reader<'a>, mark: Mark<'a>) -> ParseResult<'a, Instruction> {
    Err(mark.into_error(ErrorKind::UnknownInstruction))
}

pub const INSTRUCTIONS: [for<'a> fn(
    &mut Reader<'a>,
    mark: Mark<'a>,
) -> ParseResult<'a, Instruction>; 256] = [
    // 5.4.1 Control instructions
    // 0x00
    |_, _| Ok(Instruction::Unreachable),
    // 0x01
    |_, _| Ok(Instruction::Nop),
    // 0x02
    |reader, _mark| {
        Ok(Instruction::Block {
            bt: BlockType::parse(reader)?,
            inner: Instruction::parse_until_0x0B(reader)?,
        })
    },
    // 0x03
    |reader, _mark| {
        Ok(Instruction::Loop {
            bt: BlockType::parse(reader)?,
            inner: Instruction::parse_until_0x0B(reader)?,
        })
    },
    // 0x04
    |reader, _mark| {
        Ok({
            let bt = BlockType::parse(reader)?;
            let (if_inner, else_inner) = Instruction::parse_until_0x0B_with_else(reader)?;
            if let Some(else_inner) = else_inner {
                Instruction::IfElse {
                    bt,
                    if_inner,
                    else_inner,
                }
            } else {
                Instruction::If {
                    bt,
                    inner: if_inner,
                }
            }
        })
    },
    // 0x05 .. 0x0B
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    // 0x0C
    |reader, _mark| {
        Ok(Instruction::Branch {
            index: reader.read_index()?,
        })
    },
    // 0x0D
    |reader, _mark| {
        Ok(Instruction::BranchIf {
            index: reader.read_index()?,
        })
    },
    // 0x0E
    |reader, _mark| {
        Ok({
            let labels = reader.read_vec(|r| r.read_index())?;
            let index = reader.read_index()?;
            Instruction::BranchTable { labels, index }
        })
    },
    // 0x0F
    |_, _| Ok(Instruction::Return),
    // 0x10
    |reader, _mark| {
        Ok(Instruction::Call {
            function: reader.read_index()?,
        })
    },
    // 0x11
    |reader, _mark| {
        Ok(Instruction::CallIndirect {
            type_idx: reader.read_index()?,
            table_idx: reader.read_index()?,
        })
    },
    // 0x12 .. 0x19
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    // 5.4.3 Parametric instructions
    // 0x1A
    |_, _| Ok(Instruction::Drop),
    // 0x1B
    |_, _| Ok(Instruction::Select),
    // 0x1C
    |reader, _mark| {
        Ok(Instruction::SelectVal {
            val: reader.read_vec(ValType::parse)?,
        })
    },
    // 0x1D .. 0x1F
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    // 5.4.4 Variable instructions
    // 0x20
    |reader, _mark| Ok(Instruction::LocalGet(reader.read_index()?)),
    // 0x21
    |reader, _mark| Ok(Instruction::LocalSet(reader.read_index()?)),
    // 0x22
    |reader, _mark| Ok(Instruction::LocalTee(reader.read_index()?)),
    // 0x23
    |reader, _mark| Ok(Instruction::GlobalGet(reader.read_index()?)),
    // 0x24
    |reader, _mark| Ok(Instruction::GlobalSet(reader.read_index()?)),
    // 5.4.5 Table instructions
    // 0x25
    |reader, _mark| {
        Ok(Instruction::TableGet {
            index: reader.read_index()?,
        })
    },
    // 0x26
    |reader, _mark| {
        Ok(Instruction::TableSet {
            index: reader.read_index()?,
        })
    },
    // table.init: see parse_extended
    // table.drop: see parse_extended
    // table.copy: see parse_extended
    // table.grow: see parse_extended
    // table.size: see parse_extended
    // table.fill: see parse_extended

    // 0x27
    unknown_instruction,
    // 5.4.6 Memory instructions
    // 0x28
    |reader, _mark| {
        Ok(Instruction::Load {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x29
    |reader, _mark| {
        Ok(Instruction::Load {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x2A
    |reader, _mark| {
        Ok(Instruction::Load {
            numtype: NumType::F32,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x2B
    |reader, _mark| {
        Ok(Instruction::Load {
            numtype: NumType::F64,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x2C
    |reader, _mark| {
        Ok(Instruction::Load8 {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Signed,
        })
    },
    // 0x2D
    |reader, _mark| {
        Ok(Instruction::Load8 {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Unsigned,
        })
    },
    // 0x2E
    |reader, _mark| {
        Ok(Instruction::Load16 {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Signed,
        })
    },
    // 0x2F
    |reader, _mark| {
        Ok(Instruction::Load16 {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Unsigned,
        })
    },
    // 0x30
    |reader, _mark| {
        Ok(Instruction::Load8 {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Signed,
        })
    },
    // 0x31
    |reader, _mark| {
        Ok(Instruction::Load8 {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Unsigned,
        })
    },
    // 0x32
    |reader, _mark| {
        Ok(Instruction::Load16 {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Signed,
        })
    },
    // 0x33
    |reader, _mark| {
        Ok(Instruction::Load16 {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Unsigned,
        })
    },
    // 0x34
    |reader, _mark| {
        Ok(Instruction::Load32 {
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Signed,
        })
    },
    // 0x35
    |reader, _mark| {
        Ok(Instruction::Load32 {
            memarg: MemArg::parse(reader)?,
            signedness: Signedness::Unsigned,
        })
    },
    // 0x36
    |reader, _mark| {
        Ok(Instruction::Store {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x37
    |reader, _mark| {
        Ok(Instruction::Store {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x38
    |reader, _mark| {
        Ok(Instruction::Store {
            numtype: NumType::F32,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x39
    |reader, _mark| {
        Ok(Instruction::Store {
            numtype: NumType::F64,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x3A
    |reader, _mark| {
        Ok(Instruction::Store8 {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x3B
    |reader, _mark| {
        Ok(Instruction::Store16 {
            numtype: NumType::I32,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x3C
    |reader, _mark| {
        Ok(Instruction::Store8 {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x3D
    |reader, _mark| {
        Ok(Instruction::Store16 {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x3E
    |reader, _mark| {
        Ok(Instruction::Store32 {
            numtype: NumType::I64,
            memarg: MemArg::parse(reader)?,
        })
    },
    // 0x3F
    |reader, _mark| {
        Ok({
            let _nul = reader.read_u8()?;
            Instruction::MemorySize
        })
    },
    // 0x40
    |reader, _mark| {
        Ok({
            let _nul = reader.read_u8()?;
            Instruction::MemoryGrow
        })
    },
    // memory.init: see parse_extended
    // data.drop: see parse_extended
    // memory.copy: see parse_extended
    // memory.fill: see parse_extended

    // 5.4.7 Numeric instructions
    // 0x41
    |reader, _mark| Ok(Instruction::I32Const(reader.read_int()?)),
    // 0x42
    |reader, _mark| Ok(Instruction::I64Const(reader.read_int()?)),
    // 0x43
    |reader, _mark| Ok(Instruction::F32Const(reader.read_f32()?)),
    // 0x44
    |reader, _mark| Ok(Instruction::F64Const(reader.read_f64()?)),
    // 0x45
    |_, _| Ok(Instruction::I32EqualZero),
    // 0x46
    |_, _| Ok(Instruction::I32Equals),
    // 0x47
    |_, _| Ok(Instruction::I32NotEquals),
    // 0x48
    |_, _| Ok(Instruction::I32LessThanSigned),
    // 0x49
    |_, _| Ok(Instruction::I32LessThanUnsigned),
    // 0x4A
    |_, _| Ok(Instruction::I32GreaterThanSigned),
    // 0x4B
    |_, _| Ok(Instruction::I32GreaterThanUnsigned),
    // 0x4C
    |_, _| Ok(Instruction::I32LessOrEqualToSigned),
    // 0x4D
    |_, _| Ok(Instruction::I32LessOrEqualToUnsigned),
    // 0x4E
    |_, _| Ok(Instruction::I32GreaterOrEqualToSigned),
    // 0x4F
    |_, _| Ok(Instruction::I32GreaterOrEqualToUnsigned),
    // 0x50
    |_, _| Ok(Instruction::I64EqualZero),
    // 0x51
    |_, _| Ok(Instruction::I64Equals),
    // 0x52
    |_, _| Ok(Instruction::I64NotEquals),
    // 0x53
    |_, _| Ok(Instruction::I64LessThanSigned),
    // 0x54
    |_, _| Ok(Instruction::I64LessThanUnsigned),
    // 0x55
    |_, _| Ok(Instruction::I64GreaterThanSigned),
    // 0x56
    |_, _| Ok(Instruction::I64GreaterThanUnsigned),
    // 0x57
    |_, _| Ok(Instruction::I64LessOrEqualToSigned),
    // 0x58
    |_, _| Ok(Instruction::I64LessOrEqualToUnsigned),
    // 0x59
    |_, _| Ok(Instruction::I64GreaterOrEqualToSigned),
    // 0x5A
    |_, _| Ok(Instruction::I64GreaterOrEqualToUnsigned),
    // 0x5B
    |_, _| Ok(Instruction::F32Equals),
    // 0x5C
    |_, _| Ok(Instruction::F32NotEquals),
    // 0x5D
    |_, _| Ok(Instruction::F32LessThan),
    // 0x5E
    |_, _| Ok(Instruction::F32GreaterThan),
    // 0x5F
    |_, _| Ok(Instruction::F32LessOrEqualTo),
    // 0x60
    |_, _| Ok(Instruction::F32GreaterOrEqualTo),
    // 0x61
    |_, _| Ok(Instruction::F64Equals),
    // 0x62
    |_, _| Ok(Instruction::F64NotEquals),
    // 0x63
    |_, _| Ok(Instruction::F64LessThan),
    // 0x64
    |_, _| Ok(Instruction::F64GreaterThan),
    // 0x65
    |_, _| Ok(Instruction::F64LessOrEqualTo),
    // 0x66
    |_, _| Ok(Instruction::F64GreaterOrEqualTo),
    // 0x67
    |_, _| Ok(Instruction::I32CountLeadingZeroBits),
    // 0x68
    |_, _| Ok(Instruction::I32CountTrailingZeroBits),
    // 0x69
    |_, _| Ok(Instruction::I32CountNonZeroBits),
    // 0x6A
    |_, _| Ok(Instruction::I32Add),
    // 0x6B
    |_, _| Ok(Instruction::I32Sub),
    // 0x6C
    |_, _| Ok(Instruction::I32Mul),
    // 0x6D
    |_, _| Ok(Instruction::I32DivSigned),
    // 0x6E
    |_, _| Ok(Instruction::I32DivUnsigned),
    // 0x6F
    |_, _| Ok(Instruction::I32RemainderSigned),
    // 0x70
    |_, _| Ok(Instruction::I32RemainderUnsigned),
    // 0x71
    |_, _| Ok(Instruction::I32LogicalAnd),
    // 0x72
    |_, _| Ok(Instruction::I32LogicalOr),
    // 0x73
    |_, _| Ok(Instruction::I32LogicalXor),
    // 0x74
    |_, _| Ok(Instruction::I32ShiftLeft),
    // 0x75
    |_, _| Ok(Instruction::I32ShiftRightSigned),
    // 0x76
    |_, _| Ok(Instruction::I32ShiftRightUnsigned),
    // 0x77
    |_, _| Ok(Instruction::I32RotateLeft),
    // 0x78
    |_, _| Ok(Instruction::I32RotateRight),
    // 0x79
    |_, _| Ok(Instruction::I64CountLeadingZeroBits),
    // 0x7A
    |_, _| Ok(Instruction::I64CountTrailingZeroBits),
    // 0x7B
    |_, _| Ok(Instruction::I64CountNonZeroBits),
    // 0x7C
    |_, _| Ok(Instruction::I64Add),
    // 0x7D
    |_, _| Ok(Instruction::I64Sub),
    // 0x7E
    |_, _| Ok(Instruction::I64Mul),
    // 0x7F
    |_, _| Ok(Instruction::I64DivSigned),
    // 0x80
    |_, _| Ok(Instruction::I64DivUnsigned),
    // 0x81
    |_, _| Ok(Instruction::I64RemainderSigned),
    // 0x82
    |_, _| Ok(Instruction::I64RemainderUnsigned),
    // 0x83
    |_, _| Ok(Instruction::I64LogicalAnd),
    // 0x84
    |_, _| Ok(Instruction::I64LogicalOr),
    // 0x85
    |_, _| Ok(Instruction::I64LogicalXor),
    // 0x86
    |_, _| Ok(Instruction::I64ShiftLeft),
    // 0x87
    |_, _| Ok(Instruction::I64ShiftRightSigned),
    // 0x88
    |_, _| Ok(Instruction::I64ShiftRightUnsigned),
    // 0x89
    |_, _| Ok(Instruction::I64RotateLeft),
    // 0x8A
    |_, _| Ok(Instruction::I64RotateRight),
    // 0x8B
    |_, _| Ok(Instruction::F32Abs),
    // 0x8C
    |_, _| Ok(Instruction::F32Neg),
    // 0x8D
    |_, _| Ok(Instruction::F32Ceil),
    // 0x8E
    |_, _| Ok(Instruction::F32Floor),
    // 0x8F
    |_, _| Ok(Instruction::F32Trunc),
    // 0x90
    |_, _| Ok(Instruction::F32Nearest),
    // 0x91
    |_, _| Ok(Instruction::F32Sqrt),
    // 0x92
    |_, _| Ok(Instruction::F32Add),
    // 0x93
    |_, _| Ok(Instruction::F32Sub),
    // 0x94
    |_, _| Ok(Instruction::F32Mul),
    // 0x95
    |_, _| Ok(Instruction::F32Div),
    // 0x96
    |_, _| Ok(Instruction::F32Min),
    // 0x97
    |_, _| Ok(Instruction::F32Max),
    // 0x98
    |_, _| Ok(Instruction::F32CopySign),
    // 0x99
    |_, _| Ok(Instruction::F64Abs),
    // 0x9A
    |_, _| Ok(Instruction::F64Neg),
    // 0x9B
    |_, _| Ok(Instruction::F64Ceil),
    // 0x9C
    |_, _| Ok(Instruction::F64Floor),
    // 0x9D
    |_, _| Ok(Instruction::F64Trunc),
    // 0x9E
    |_, _| Ok(Instruction::F64Nearest),
    // 0x9F
    |_, _| Ok(Instruction::F64Sqrt),
    // 0xA0
    |_, _| Ok(Instruction::F64Add),
    // 0xA1
    |_, _| Ok(Instruction::F64Sub),
    // 0xA2
    |_, _| Ok(Instruction::F64Mul),
    // 0xA3
    |_, _| Ok(Instruction::F64Div),
    // 0xA4
    |_, _| Ok(Instruction::F64Min),
    // 0xA5
    |_, _| Ok(Instruction::F64Max),
    // 0xA6
    |_, _| Ok(Instruction::F64CopySign),
    // 0xA7
    |_, _| Ok(Instruction::I32WrapI64),
    // 0xA8
    |_, _| Ok(Instruction::I32TruncF32Signed),
    // 0xA9
    |_, _| Ok(Instruction::I32TruncF32Unsigned),
    // 0xAA
    |_, _| Ok(Instruction::I32TruncF64Signed),
    // 0xAB
    |_, _| Ok(Instruction::I32TruncF64Unsigned),
    // 0xAC
    |_, _| Ok(Instruction::I64ExtendI32Signed),
    // 0xAD
    |_, _| Ok(Instruction::I64ExtendI32Unsigned),
    // 0xAE
    |_, _| Ok(Instruction::I64TruncF32Signed),
    // 0xAF
    |_, _| Ok(Instruction::I64TruncF32Unsigned),
    // 0xB0
    |_, _| Ok(Instruction::I64TruncF64Signed),
    // 0xB1
    |_, _| Ok(Instruction::I64TruncF64Unsigned),
    // 0xB2
    |_, _| Ok(Instruction::F32ConvertI32Signed),
    // 0xB3
    |_, _| Ok(Instruction::F32ConvertI32Unsigned),
    // 0xB4
    |_, _| Ok(Instruction::F32ConvertI64Signed),
    // 0xB5
    |_, _| Ok(Instruction::F32ConvertI64Unsigned),
    // 0xB6
    |_, _| Ok(Instruction::F32DemoteF64),
    // 0xB7
    |_, _| Ok(Instruction::F64ConvertI32Signed),
    // 0xB8
    |_, _| Ok(Instruction::F64ConvertI32Unsigned),
    // 0xB9
    |_, _| Ok(Instruction::F64ConvertI64Signed),
    // 0xBA
    |_, _| Ok(Instruction::F64ConvertI64Unsigned),
    // 0xBB
    |_, _| Ok(Instruction::F64PromoteF32),
    // 0xBC
    |_, _| Ok(Instruction::I32ReinterpretAsF32),
    // 0xBD
    |_, _| Ok(Instruction::I64ReinterpretAsF64),
    // 0xBE
    |_, _| Ok(Instruction::F32ReinterpretAsI32),
    // 0xBF
    |_, _| Ok(Instruction::F64ReinterpretAsI64),
    // 0xC0
    |_, _| Ok(Instruction::I32Extend8Signed),
    // 0xC1
    |_, _| Ok(Instruction::I32Extend16Signed),
    // 0xC2
    |_, _| Ok(Instruction::I64Extend8Signed),
    // 0xC3
    |_, _| Ok(Instruction::I64Extend16Signed),
    // 0xC4
    |_, _| Ok(Instruction::I64Extend32Signed),
    // 0xC5 .. 0xCF
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    // 5.4.2 Reference instructions
    // 0xD0
    |reader, _mark| {
        Ok({
            let reftype = RefType::parse(reader)?;
            Instruction::RefNull { reftype }
        })
    },
    // 0xD1
    |_, _| Ok(Instruction::RefIsNull),
    // 0xD2
    |reader, _mark| {
        Ok({
            let index = reader.read_index()?;
            Instruction::RefFunc { index }
        })
    },
    // 0xD3 .. 0xDF
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    // 0xE0 .. 0xEF
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    // 0xF0 .. 0xFB
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    unknown_instruction,
    // 0xFC
    Instruction::parse_extended,
    // 0xFD
    Instruction::parse_vector,
    // 0xFE .. 0xFF
    unknown_instruction,
    unknown_instruction,
];
