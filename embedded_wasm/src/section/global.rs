use super::ValType;
use crate::{instruction::Instruction, ParseResult, Reader, Vec};

#[derive(Debug, Clone)]
pub struct Global {
    pub r#type: ValType,
    pub mutable: bool,
    pub init: Vec<Instruction>,
}

impl Global {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let r#type = ValType::parse(reader)?;
        let mutable = reader.read_u8()? == 0x01;
        let init = Instruction::parse_until_0x0B(reader)?;
        Ok(Global {
            r#type,
            mutable,
            init,
        })
    }
}
