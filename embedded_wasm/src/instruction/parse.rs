use super::{BlockType, Instruction, MemArg};
use crate::{
    section::{RefType, ValType},
    ErrorKind, ParseResult, Reader, Vec,
};

impl Instruction {
    #[allow(non_snake_case)]
    pub fn parse_until_0x0B<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Vec<Instruction>> {
        let mut instructions = Vec::new();
        while !reader.read_u8_if(|b| b == 0x0B)? {
            instructions.push(Instruction::parse(reader)?);
        }
        Ok(instructions)
    }

    #[allow(non_snake_case)]
    fn parse_until_0x0B_with_else<'a>(
        reader: &mut Reader<'a>,
    ) -> ParseResult<'a, (Vec<Instruction>, Option<Vec<Instruction>>)> {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        enum IfOrElse {
            If,
            Else,
        }
        let mut if_inner = Vec::new();
        let mut else_inner = Vec::new();
        let mut step = IfOrElse::If;
        while !reader.read_u8_if(|b| b == 0x0B)? {
            if reader.read_u8_if(|b| b == 0x05)? {
                if step == IfOrElse::Else {
                    return Err(reader.mark_relative(-1).to_error(ErrorKind::DuplicateElse));
                }
                step = IfOrElse::Else;
            } else {
                if step == IfOrElse::If {
                    &mut if_inner
                } else {
                    &mut else_inner
                }
                .push(Instruction::parse(reader)?);
            }
        }

        Ok((
            if_inner,
            if step == IfOrElse::If {
                None
            } else {
                Some(else_inner)
            },
        ))
    }

    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        Ok(match reader.read_u8()? {
            // 5.4.1 Control instructions
            0x00 => Self::Unreachable,
            0x01 => Self::Nop,
            0x02 => Self::Block {
                bt: BlockType::parse(reader)?,
                inner: Self::parse_until_0x0B(reader)?,
            },
            0x03 => Self::Loop {
                bt: BlockType::parse(reader)?,
                inner: Self::parse_until_0x0B(reader)?,
            },
            0x04 => {
                let bt = BlockType::parse(reader)?;
                let (if_inner, else_inner) = Self::parse_until_0x0B_with_else(reader)?;
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
            }
            0x0C => Self::Branch {
                index: reader.read_index()?,
            },
            0x0D => Self::BranchIf {
                index: reader.read_index()?,
            },
            0x0E => {
                let labels = reader.read_vec(|r| r.read_index())?;
                let index = reader.read_index()?;
                Instruction::BranchTable { labels, index }
            }
            0x0F => Instruction::Return,
            0x10 => Self::Call {
                function: reader.read_index()?,
            },
            0x11 => Self::CallIndirect {
                type_idx: reader.read_index()?,
                table_idx: reader.read_index()?,
            },

            // 5.4.2 Reference instructions
            0xD0 => {
                let reftype = RefType::parse(reader)?;
                Instruction::RefNull { reftype }
            }
            0xD1 => Instruction::RefIsNull,
            0xD2 => {
                let index = reader.read_index()?;
                Instruction::RefFunc { index }
            }

            // 5.4.3 Parametric instructions
            0x1A => Instruction::Drop,
            0x1B => Instruction::Select,
            0x1C => Instruction::SelectVal {
                val: reader.read_vec(ValType::parse)?,
            },

            // 5.4.4 Variable instructions
            0x20 => Self::LocalGet(reader.read_index()?),
            0x21 => Self::LocalSet(reader.read_index()?),
            0x22 => Self::LocalTee(reader.read_index()?),
            0x23 => Self::GlobalGet(reader.read_index()?),
            0x24 => Self::GlobalSet(reader.read_index()?),

            // 5.4.5 Table instructions
            0x25 => Self::TableGet {
                index: reader.read_index()?,
            },
            0x26 => Self::TableSet {
                index: reader.read_index()?,
            },
            // table.init: see parse_extended
            // table.drop: see parse_extended
            // table.copy: see parse_extended
            // table.grow: see parse_extended
            // table.size: see parse_extended
            // table.fill: see parse_extended

            // 5.4.6 Memory instructions
            0x28 => Self::I32Load(MemArg::parse(reader)?),
            0x29 => Self::I64Load(MemArg::parse(reader)?),
            0x2A => Self::F32Load(MemArg::parse(reader)?),
            0x2B => Self::F64Load(MemArg::parse(reader)?),
            0x2C => Self::I32Load8S(MemArg::parse(reader)?),
            0x2D => Self::I32Load8U(MemArg::parse(reader)?),
            0x2E => Self::I32Load16S(MemArg::parse(reader)?),
            0x2F => Self::I32Load16U(MemArg::parse(reader)?),
            0x30 => Self::I64Load8S(MemArg::parse(reader)?),
            0x31 => Self::I64Load8U(MemArg::parse(reader)?),
            0x32 => Self::I64Load16S(MemArg::parse(reader)?),
            0x33 => Self::I64Load16U(MemArg::parse(reader)?),
            0x34 => Self::I64Load32S(MemArg::parse(reader)?),
            0x35 => Self::I64Load32U(MemArg::parse(reader)?),
            0x36 => Self::I32Store(MemArg::parse(reader)?),
            0x37 => Self::I64Store(MemArg::parse(reader)?),
            0x38 => Self::F32Store(MemArg::parse(reader)?),
            0x39 => Self::F64Store(MemArg::parse(reader)?),
            0x3A => Self::I32Store8(MemArg::parse(reader)?),
            0x3B => Self::I32Store16(MemArg::parse(reader)?),
            0x3C => Self::I64Store8(MemArg::parse(reader)?),
            0x3D => Self::I64Store16(MemArg::parse(reader)?),
            0x3E => Self::I64Store32(MemArg::parse(reader)?),
            0x3F => {
                let _nul = reader.read_u8()?;
                Instruction::MemorySize
            }
            0x40 => {
                let _nul = reader.read_u8()?;
                Instruction::MemoryGrow
            }
            // memory.init: see parse_extended
            // data.drop: see parse_extended
            // memory.copy: see parse_extended
            // memory.fill: see parse_extended

            // 5.4.7 Numeric instructions
            0x41 => Self::I32Const(reader.read_int()?),
            0x42 => Self::I64Const(reader.read_int()?),
            0x43 => Self::F32Const(reader.read_f32()?),
            0x44 => Self::F64Const(reader.read_f64()?),
            0x45 => Self::I32EqualZero,
            0x46 => Self::I32Equals,
            0x47 => Self::I32NotEquals,
            0x48 => Self::I32LessThanSigned,
            0x49 => Self::I32LessThanUnsigned,
            0x4A => Self::I32GreaterThanSigned,
            0x4B => Self::I32GreaterThanUnsigned,
            0x4C => Self::I32LessOrEqualToSigned,
            0x4D => Self::I32LessOrEqualToUnsigned,
            0x4E => Self::I32GreaterOrEqualToSigned,
            0x4F => Self::I32GreaterOrEqualToUnsigned,

            0x50 => Self::I64EqualZero,
            0x51 => Self::I64Equals,
            0x52 => Self::I64NotEquals,
            0x53 => Self::I64LessThanSigned,
            0x54 => Self::I64LessThanUnsigned,
            0x55 => Self::I64GreaterThanSigned,
            0x56 => Self::I64GreaterThanUnsigned,
            0x57 => Self::I64LessOrEqualToSigned,
            0x58 => Self::I64LessOrEqualToUnsigned,
            0x59 => Self::I64GreaterOrEqualToSigned,
            0x5A => Self::I64GreaterOrEqualToUnsigned,

            0x5B => Self::F32Equals,
            0x5C => Self::F32NotEquals,
            0x5D => Self::F32LessThan,
            0x5E => Self::F32GreaterThan,
            0x5F => Self::F32LessOrEqualTo,
            0x60 => Self::F32GreaterOrEqualTo,

            0x61 => Self::F64Equals,
            0x62 => Self::F64NotEquals,
            0x63 => Self::F64LessThan,
            0x64 => Self::F64GreaterThan,
            0x65 => Self::F64LessOrEqualTo,
            0x66 => Self::F64GreaterOrEqualTo,

            0x67 => Self::I32CountLeadingZeroBits,
            0x68 => Self::I32CountTrailingZeroBits,
            0x69 => Self::I32CountNonZeroBits,
            0x6A => Self::I32Add,
            0x6B => Self::I32Sub,
            0x6C => Self::I32Mul,
            0x6D => Self::I32DivSigned,
            0x6E => Self::I32DivUnsigned,
            0x6F => Self::I32RemainderSigned,
            0x70 => Self::I32RemainderUnsigned,
            0x71 => Self::I32LogicalAnd,
            0x72 => Self::I32LogicalOr,
            0x73 => Self::I32LogicalXor,
            0x74 => Self::I32ShiftLeft,
            0x75 => Self::I32ShiftRightSigned,
            0x76 => Self::I32ShiftRightUnsigned,
            0x77 => Self::I32RotateLeft,
            0x78 => Self::I32RotateRight,

            0x79 => Self::I64CountLeadingZeroBits,
            0x7A => Self::I64CountTrailingZeroBits,
            0x7B => Self::I64CountNonZeroBits,
            0x7C => Self::I64Add,
            0x7D => Self::I64Sub,
            0x7E => Self::I64Mul,
            0x7F => Self::I64DivSigned,
            0x80 => Self::I64DivUnsigned,
            0x81 => Self::I64RemainderSigned,
            0x82 => Self::I64RemainderUnsigned,
            0x83 => Self::I64LogicalAnd,
            0x84 => Self::I64LogicalOr,
            0x85 => Self::I64LogicalXor,
            0x86 => Self::I64ShiftLeft,
            0x87 => Self::I64ShiftRightSigned,
            0x88 => Self::I64ShiftRightUnsigned,
            0x89 => Self::I64RotateLeft,
            0x8A => Self::I64RotateRight,

            0x8B => Self::F32Abs,
            0x8C => Self::F32Neg,
            0x8D => Self::F32Ceil,
            0x8E => Self::F32Floor,
            0x8F => Self::F32Trunc,
            0x90 => Self::F32Nearest,
            0x91 => Self::F32Sqrt,
            0x92 => Self::F32Add,
            0x93 => Self::F32Sub,
            0x94 => Self::F32Mul,
            0x95 => Self::F32Div,
            0x96 => Self::F32Min,
            0x97 => Self::F32Max,
            0x98 => Self::F32CopySign,

            0x99 => Self::F64Abs,
            0x9A => Self::F64Neg,
            0x9B => Self::F64Ceil,
            0x9C => Self::F64Floor,
            0x9D => Self::F64Trunc,
            0x9E => Self::F64Nearest,
            0x9F => Self::F64Sqrt,
            0xA0 => Self::F64Add,
            0xA1 => Self::F64Sub,
            0xA2 => Self::F64Mul,
            0xA3 => Self::F64Div,
            0xA4 => Self::F64Min,
            0xA5 => Self::F64Max,
            0xA6 => Self::F64CopySign,

            0xA7 => Self::I32WrapI64,
            0xA8 => Self::I32TruncF32Signed,
            0xA9 => Self::I32TruncF32Unsigned,
            0xAA => Self::I32TruncF64Signed,
            0xAB => Self::I32TruncF64Unsigned,

            0xAC => Self::I64ExtendI32Signed,
            0xAD => Self::I64ExtendI32Unsigned,
            0xAE => Self::I64TruncF32Signed,
            0xAF => Self::I64TruncF32Unsigned,
            0xB0 => Self::I64TruncF64Signed,
            0xB1 => Self::I64TruncF64Unsigned,

            0xB2 => Self::F32ConvertI32Signed,
            0xB3 => Self::F32ConvertI32Unsigned,
            0xB4 => Self::F32ConvertI64Signed,
            0xB5 => Self::F32ConvertI64Unsigned,
            0xB6 => Self::F32DemoteF64,

            0xB7 => Self::F64ConvertI32Signed,
            0xB8 => Self::F64ConvertI32Unsigned,
            0xB9 => Self::F64ConvertI64Signed,
            0xBA => Self::F64ConvertI64Unsigned,
            0xBB => Self::F64PromoteF32,

            0xBC => Self::I32ReinterpretAsF32,
            0xBD => Self::I64ReinterpretAsF64,
            0xBE => Self::F32ReinterpretAsI32,
            0xBF => Self::F64ReinterpretAsI64,

            0xC0 => Self::I32Extend8Signed,
            0xC1 => Self::I32Extend16Signed,
            0xC2 => Self::I64Extend8Signed,
            0xC3 => Self::I64Extend16Signed,
            0xC4 => Self::I64Extend32Signed,

            0xFC => Self::parse_extended(reader)?,

            _ => return Err(mark.to_error(ErrorKind::UnknownInstruction)),
        })
    }

    fn parse_extended<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        Ok(match reader.read_int::<u32>()? {
            0 => panic!("i32.trunc_<sat>_f32_signed, next: {:?}", reader.read_u8()),
            1 => panic!("i32.trunc_<sat>_f32_unsigned, next: {:?}", reader.read_u8()),
            2 => panic!("i32.trunc_<sat>_f64_signed, next: {:?}", reader.read_u8()),
            3 => panic!("i32.trunc_<sat>_f64_unsigned, next: {:?}", reader.read_u8()),
            4 => panic!("i64.trunc_<sat>_f32_signed, next: {:?}", reader.read_u8()),
            5 => panic!("i64.trunc_<sat>_f32_unsigned, next: {:?}", reader.read_u8()),
            6 => panic!("i64.trunc_<sat>_f64_signed, next: {:?}", reader.read_u8()),
            7 => panic!("i64.trunc_<sat>_f64_unsigned, next: {:?}", reader.read_u8()),
            8 => {
                let index = reader.read_index()?;
                let _nul = reader.read_u8()?;
                Self::MemoryInit { index }
            }
            9 => Self::DataDrop {
                index: reader.read_index()?,
            },
            10 => {
                let _nul = reader.read_u8()?;
                let _nul = reader.read_u8()?;
                Self::MemoryCopy
            }
            11 => {
                let _nul = reader.read_u8()?;
                Self::MemoryFill
            }
            12 => Self::TableInit {
                y: reader.read_index()?,
                x: reader.read_index()?,
            },
            13 => Self::TableDrop {
                x: reader.read_index()?,
            },
            14 => Self::TableCopy {
                x: reader.read_index()?,
                y: reader.read_index()?,
            },
            15 => Self::TableGrow {
                x: reader.read_index()?,
            },
            16 => Self::TableSize {
                x: reader.read_index()?,
            },
            17 => Self::TableFill {
                x: reader.read_index()?,
            },
            _ => return Err(mark.to_error(ErrorKind::UnknownExtendedInstruction)),
        })
    }
    pub fn parse_vec<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Vec<Self>> {
        let mut result = Vec::with_capacity(reader.remaining().len());
        while !reader.is_empty() {
            result.push(Self::parse(reader)?);
        }
        result.shrink_to_fit();
        Ok(result)
    }
}
