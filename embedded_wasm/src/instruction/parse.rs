use super::{BlockType, Instruction, MemArg, NumType, RefType, SectionType, Signedness, ValType};
use crate::{instruction::LaneIdx, ErrorKind, ParseResult, Reader, Vec};

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
                    return Err(reader
                        .mark_relative(-1)
                        .into_error(ErrorKind::DuplicateElse));
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
            0x28 => Self::Load {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
            },
            0x29 => Self::Load {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
            },
            0x2A => Self::Load {
                numtype: NumType::F32,
                memarg: MemArg::parse(reader)?,
            },
            0x2B => Self::Load {
                numtype: NumType::F64,
                memarg: MemArg::parse(reader)?,
            },
            0x2C => Self::Load8 {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Signed,
            },
            0x2D => Self::Load8 {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Unsigned,
            },
            0x2E => Self::Load16 {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Signed,
            },
            0x2F => Self::Load16 {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Unsigned,
            },
            0x30 => Self::Load8 {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Signed,
            },
            0x31 => Self::Load8 {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Unsigned,
            },
            0x32 => Self::Load16 {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Signed,
            },
            0x33 => Self::Load16 {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Unsigned,
            },
            0x34 => Self::Load32 {
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Signed,
            },
            0x35 => Self::Load32 {
                memarg: MemArg::parse(reader)?,
                signedness: Signedness::Unsigned,
            },
            0x36 => Self::Store {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
            },
            0x37 => Self::Store {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
            },
            0x38 => Self::Store {
                numtype: NumType::F32,
                memarg: MemArg::parse(reader)?,
            },
            0x39 => Self::Store {
                numtype: NumType::F64,
                memarg: MemArg::parse(reader)?,
            },
            0x3A => Self::Store8 {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
            },
            0x3B => Self::Store16 {
                numtype: NumType::I32,
                memarg: MemArg::parse(reader)?,
            },
            0x3C => Self::Store8 {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
            },
            0x3D => Self::Store16 {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
            },
            0x3E => Self::Store32 {
                numtype: NumType::I64,
                memarg: MemArg::parse(reader)?,
            },
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
            0xFD => Self::parse_vector(reader)?,

            _ => return Err(mark.into_error(ErrorKind::UnknownInstruction)),
        })
    }

    fn parse_extended<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        Ok(match reader.read_int::<u32>()? {
            0 => unimplemented!("i32.trunc_<sat>_f32_signed, next: {:?}", reader.read_u8()),
            1 => unimplemented!("i32.trunc_<sat>_f32_unsigned, next: {:?}", reader.read_u8()),
            2 => unimplemented!("i32.trunc_<sat>_f64_signed, next: {:?}", reader.read_u8()),
            3 => unimplemented!("i32.trunc_<sat>_f64_unsigned, next: {:?}", reader.read_u8()),
            4 => unimplemented!("i64.trunc_<sat>_f32_signed, next: {:?}", reader.read_u8()),
            5 => unimplemented!("i64.trunc_<sat>_f32_unsigned, next: {:?}", reader.read_u8()),
            6 => unimplemented!("i64.trunc_<sat>_f64_signed, next: {:?}", reader.read_u8()),
            7 => unimplemented!("i64.trunc_<sat>_f64_unsigned, next: {:?}", reader.read_u8()),
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
            _ => return Err(mark.into_error(ErrorKind::UnknownExtendedInstruction)),
        })
    }

    fn parse_vector<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        use super::{Signedness::*, VectorInstruction::*};
        let mark = reader.mark();
        Ok(Self::Vector(match reader.read_int::<u32>()? {
            0 => V128Load(MemArg::parse(reader)?),
            1 => V128Load8x8(MemArg::parse(reader)?, Signed),
            2 => V128Load8x8(MemArg::parse(reader)?, Unsigned),
            3 => V128Load16x4(MemArg::parse(reader)?, Signed),
            4 => V128Load16x4(MemArg::parse(reader)?, Unsigned),
            5 => V128Load32x2(MemArg::parse(reader)?, Signed),
            6 => V128Load32x2(MemArg::parse(reader)?, Unsigned),
            7 => V128Load8Splat(MemArg::parse(reader)?),
            8 => V128Load16Splat(MemArg::parse(reader)?),
            9 => V128Load32Splat(MemArg::parse(reader)?),
            10 => V128Load64Splat(MemArg::parse(reader)?),

            11 => V128Store(MemArg::parse(reader)?),
            12 => V128Const(reader.read_exact()?),
            13 => {
                let slice: [u8; 16] = reader.read_exact()?;
                let mut lanes = [LaneIdx::default(); 16];
                for (idx, lane) in lanes.iter_mut().enumerate() {
                    *lane = LaneIdx(slice[idx]);
                    if lane.0 >= 16 {
                        return Err(reader
                            .mark_relative(-(16 - idx as isize))
                            .into_error(ErrorKind::InvalidLaneIndex { max: 16 }));
                    }
                }
                I8x16Shuffle(lanes)
            }
            14 => I8x16Swizzle,
            15 => I8x16Splat,
            16 => I16x8Splat,
            17 => I32x4Splat,
            18 => I64x2Splat,
            19 => F32x4Splat,
            20 => F64x2Splat,

            21 => I8x16ExtractLane(LaneIdx::parse_max_16(reader)?, Signed),
            22 => I8x16ExtractLane(LaneIdx::parse_max_16(reader)?, Unsigned),
            23 => I8x16ReplaceLane(LaneIdx::parse_max_16(reader)?),
            24 => I16x8ExtractLane(LaneIdx::parse_max_8(reader)?, Signed),
            25 => I16x8ExtractLane(LaneIdx::parse_max_8(reader)?, Unsigned),
            26 => I16x8ReplaceLane(LaneIdx::parse_max_8(reader)?),
            27 => I32x4ExtractLane(LaneIdx::parse_max_4(reader)?),
            28 => I32x4ReplaceLane(LaneIdx::parse_max_4(reader)?),
            29 => I64x2ExtractLane(LaneIdx::parse_max_2(reader)?),

            30 => I64x2ReplaceLane(LaneIdx::parse_max_2(reader)?),
            31 => F32x4ExtractLane(LaneIdx::parse_max_4(reader)?),
            32 => F32x4ReplaceLane(LaneIdx::parse_max_4(reader)?),
            33 => F64x2ExtractLane(LaneIdx::parse_max_2(reader)?),
            34 => F64x2ReplaceLane(LaneIdx::parse_max_2(reader)?),
            35 => I8x16Equal,
            36 => I8x16NotEqual,
            37 => I8x16LessThan(Signed),
            38 => I8x16LessThan(Unsigned),
            39 => I8x16GreaterThan(Signed),

            40 => I8x16GreaterThan(Unsigned),
            41 => I8x16LessOrEqualTo(Signed),
            42 => I8x16LessOrEqualTo(Unsigned),
            43 => I8x16GreaterOrEqualTo(Signed),
            44 => I8x16GreaterOrEqualTo(Unsigned),
            45 => I16x8Equal,
            46 => I16x8NotEqual,
            47 => I16x8LessThan(Signed),
            48 => I16x8LessThan(Unsigned),
            49 => I16x8GreaterThan(Signed),

            50 => I16x8GreaterThan(Unsigned),
            51 => I16x8LessOrEqualTo(Signed),
            52 => I16x8LessOrEqualTo(Unsigned),
            53 => I16x8GreaterOrEqualTo(Signed),
            54 => I16x8GreaterOrEqualTo(Unsigned),
            55 => I32x4Equal,
            56 => I32x4NotEqual,
            57 => I32x4LessThan(Signed),
            58 => I32x4LessThan(Unsigned),
            59 => I32x4GreaterThan(Signed),

            60 => I32x4GreaterThan(Unsigned),
            61 => I32x4LessOrEqualTo(Signed),
            62 => I32x4LessOrEqualTo(Unsigned),
            63 => I32x4GreaterOrEqualTo(Signed),
            64 => I32x4GreaterOrEqualTo(Unsigned),
            65 => F32x4Equal,
            66 => F32x4NotEqual,
            67 => F32x4LessThan,
            68 => F32x4GreaterThan,
            69 => F32x4LessOrEqualTo,

            70 => F32x4GreaterOrEqualTo,
            71 => F64x2Equal,
            72 => F64x2NotEqual,
            73 => F64x2LessThan,
            74 => F64x2GreaterThan,
            75 => F64x2LessOrEqualTo,
            76 => F64x2GreaterOrEqualTo,
            77 => V128Not,
            78 => V128And,
            79 => V128AndNot,

            80 => V128Or,
            81 => V128Xor,
            82 => V128BitSelect,
            83 => V128AnyTrue,
            84 => V128Load8Lane(MemArg::parse(reader)?, LaneIdx::parse_max_16(reader)?),
            85 => V128Load16Lane(MemArg::parse(reader)?, LaneIdx::parse_max_8(reader)?),
            86 => V128Load32Lane(MemArg::parse(reader)?, LaneIdx::parse_max_4(reader)?),
            87 => V128Load64Lane(MemArg::parse(reader)?, LaneIdx::parse_max_2(reader)?),
            88 => V128Store8Lane(MemArg::parse(reader)?, LaneIdx::parse_max_16(reader)?),
            89 => V128Store16Lane(MemArg::parse(reader)?, LaneIdx::parse_max_8(reader)?),

            90 => V128Store32Lane(MemArg::parse(reader)?, LaneIdx::parse_max_4(reader)?),
            91 => V128Store64Lane(MemArg::parse(reader)?, LaneIdx::parse_max_2(reader)?),
            92 => V128Load32Zero(MemArg::parse(reader)?),
            93 => V128Load64Zero(MemArg::parse(reader)?),
            94 => F32x4DemoteF64x2Zero,
            95 => F64x2PromoteLowF32x4,
            96 => I8x16Abs,
            97 => I8x16Neg,
            98 => I8x16PopCnt,
            99 => I8x16AllTrue,

            100 => I8x16Bitmask,
            101 => I8x16NarrowI16x8(Signed),
            102 => I8x16NarrowI16x8(Signed),
            103 => F32x4Ceil,
            104 => F32x4Floor,
            105 => F32x4Trunc,
            106 => F32x4Nearest,
            107 => I8x16ShiftLeft,
            108 => I8x16ShiftRight(Signed),
            109 => I8x16ShiftRight(Unsigned),

            110 => I8x16Add,
            111 => I8x16AddSaturating(Signed),
            112 => I8x16AddSaturating(Unsigned),
            113 => I8x16Sub,
            114 => I8x16SubSaturating(Signed),
            115 => I8x16SubSaturating(Unsigned),
            116 => F64x2Ceil,
            117 => F64x2Floor,
            118 => I8x16Min(Signed),
            119 => I8x16Min(Unsigned),

            120 => I8x16Max(Signed),
            121 => I8x16Max(Unsigned),
            122 => F64x2Trunc,
            123 => I8x16Average,
            124 => I16x8ExtAddPairWiseI8x16(Signed),
            125 => I16x8ExtAddPairWiseI8x16(Unsigned),
            126 => I32x4ExtAddPairwiseI16x8(Signed),
            127 => I32x4ExtAddPairwiseI16x8(Unsigned),
            128 => I16x8Abs,
            129 => I16x8Neg,

            130 => I16x8Q16MulrSat,
            131 => I16x8AllTrue,
            132 => I16x8Bitmask,
            133 => I16x8NarrowI32x4(Signed),
            134 => I16x8NarrowI32x4(Unsigned),
            135 => I16x8ExtendLowI8x16(Signed),
            136 => I16x8ExtendHighI8x16(Signed),
            137 => I16x8ExtendLowI8x16(Unsigned),
            138 => I16x8ExtendHighI8x16(Unsigned),
            139 => I16x8ShiftLeft,

            140 => I16x8ShiftRight(Signed),
            141 => I16x8ShiftRight(Unsigned),
            142 => I16x8Add,
            143 => I16x8AddSaturating(Signed),
            144 => I16x8AddSaturating(Unsigned),
            145 => I16x8Sub,
            146 => I16x8SubSaturating(Signed),
            147 => I16x8SubSaturating(Unsigned),
            148 => F64x2Nearest,
            149 => I16x8Mul,

            150 => I16x8Min(Signed),
            151 => I16x8Min(Unsigned),
            152 => I16x8Max(Signed),
            153 => I16x8Max(Unsigned),

            155 => I16x8Average(Unsigned),
            156 => I16x8ExtMulLowI8x16(Signed),
            157 => I16x8ExtMulHighI8x16(Signed),
            158 => I16x8ExtMulLowI8x16(Unsigned),
            159 => I16x8ExtMulHighI8x16(Unsigned),

            160 => I32x4Abs,
            161 => I32x4Neg,

            163 => I32x4AllTrue,
            164 => I32x4Bitmask,

            167 => I32x4ExtendLowI16x8(Signed),
            168 => I32x4ExtendHighI16x8(Signed),
            169 => I32x4ExtendLowI16x8(Unsigned),

            170 => I32x4ExtendHighI16x8(Unsigned),
            171 => I32x4ShiftLeft,
            172 => I32x4ShiftRight(Signed),
            173 => I32x4ShiftRight(Unsigned),
            174 => I32x4Add,

            177 => I32x4Sub,

            181 => I32x4Mul,
            182 => I32x4Min(Signed),
            183 => I32x4Min(Unsigned),
            184 => I32x4Max(Signed),
            185 => I32x4Max(Unsigned),
            186 => I32x4DotI16x8,

            188 => I32x4ExtMulLowI16x8(Signed),
            189 => I32x4ExtMulHighI16x8(Signed),

            190 => I32x4ExtMulLowI16x8(Unsigned),
            191 => I32x4ExtMulHighI16x8(Unsigned),
            192 => I64x2Abs,
            193 => I64x2Neg,

            195 => I64x2AllTrue,
            196 => I64x2Bitmask,
            199 => I64x2ExtendLowI32x4(Signed),

            200 => I64x2ExtendHighI32x4(Signed),
            201 => I64x2ExtendLowI32x4(Unsigned),
            202 => I64x2ExtendHighI32x4(Unsigned),
            203 => I64x2ShiftLeft,
            204 => I64x2ShiftRight(Signed),
            205 => I64x2ShiftRight(Unsigned),
            206 => I64x2Add,

            209 => I64x2Sub,

            213 => I64x2Mul,
            214 => I64x2Equal,
            215 => I64x2NotEqual,
            216 => I64x2LessThan,
            217 => I64x2GreaterThan,
            218 => I64x2LessOrEqualTo,
            219 => I64x2GreaterOrEqualTo,

            220 => I64x2ExtMulLowI32x4(Signed),
            221 => I64x2ExtMulHighI32x4(Signed),
            222 => I64x2ExtMulLowI32x4(Unsigned),
            223 => I64x2ExtMulHighI32x4(Unsigned),
            224 => F32x4Abs,
            225 => F32x4Neg,

            227 => F32x4Sqrt,
            228 => F32x4Add,
            229 => F32x4Sub,

            230 => F32x4Mul,
            231 => F32x4Div,
            232 => F32x4Min,
            233 => F32x4Max,
            234 => F32x4PMin,
            235 => F32x4PMax,
            236 => F64x2Abs,
            237 => F64x2Neg,

            239 => F64x2Sqrt,

            240 => F64x2Add,
            241 => F64x2Add,
            242 => F64x2Mul,
            243 => F64x2Div,
            244 => F64x2Min,
            245 => F64x2Max,
            246 => F64x2PMin,
            247 => F64x2PMax,
            248 => I32x4TruncSatF32x4(Signed),
            249 => I32x4TruncSatF32x4(Unsigned),

            250 => F32x4ConvertI32x4(Signed),
            251 => F32x4ConvertI32x4(Unsigned),
            252 => I32x4TruncSatF64x2Zero(Signed),
            253 => I32x4TruncSatF64x2Zero(Unsigned),
            254 => F64x2ConvertLowI32x4(Signed),
            255 => F64x2ConvertLowI32x4(Unsigned),

            _ => return Err(mark.into_error(ErrorKind::UnknownVectorInstruction)),
        }))
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

impl BlockType {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Ok(match reader.read_u8()? {
            0x40 => Self::Empty,
            x => panic!("Unknown blocktype 0x{:02X}", x),
        })
    }
}

impl MemArg {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let align = reader.read_int()?;
        let offset = reader.read_int()?;
        Ok(Self { align, offset })
    }
}

impl ValType {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        let val = reader.read_u8()?;
        Self::from_u8(val).map_err(|kind| mark.into_error(kind))
    }
    fn from_u8(val: u8) -> Result<Self, ErrorKind> {
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
impl RefType {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        let val = reader.read_u8()?;
        Self::from_u8(val).map_err(|e| mark.into_error(e))
    }
    fn from_u8(val: u8) -> core::result::Result<Self, ErrorKind> {
        Ok(match val {
            0x70 => Self::FuncRef,
            0x6F => Self::ExternRef,
            _ => return Err(ErrorKind::UnknownRefType),
        })
    }
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

impl LaneIdx {
    fn parse_max<'a, const N: u8>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        let val = reader.read_u8()?;
        if val >= N {
            Err(mark.into_error(ErrorKind::InvalidLaneIndex { max: N }))
        } else {
            Ok(Self(val))
        }
    }

    pub fn parse_max_16<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Self::parse_max::<16>(reader)
    }

    pub fn parse_max_8<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Self::parse_max::<8>(reader)
    }

    pub fn parse_max_4<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Self::parse_max::<4>(reader)
    }

    pub fn parse_max_2<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Self::parse_max::<2>(reader)
    }
}
