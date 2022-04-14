mod base_instruction_map;
mod vector_instruction_map;

use super::{BlockType, Instruction, MemArg, NumType, RefType, SectionType, ValType};
use crate::{instruction::LaneIdx, ErrorKind, Mark, ParseResult, Reader, Vec};

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
        let idx = reader.read_u8()? as usize;
        (base_instruction_map::INSTRUCTIONS[idx])(reader, mark)
    }

    fn parse_extended<'a>(reader: &mut Reader<'a>, _mark: Mark<'a>) -> ParseResult<'a, Self> {
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

    fn parse_vector<'a>(reader: &mut Reader<'a>, _mark: Mark<'a>) -> ParseResult<'a, Self> {
        let mark = reader.mark();
        let idx = reader.read_int::<u32>()?;
        if idx > u8::max_value() as u32 {
            Err(mark.into_error(ErrorKind::UnknownVectorInstruction))
        } else {
            (vector_instruction_map::INSTRUCTIONS[idx as usize])(reader, mark)
                .map(Instruction::Vector)
        }
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
