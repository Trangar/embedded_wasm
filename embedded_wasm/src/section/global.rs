use super::{Instruction, ValType};
use crate::{ParseResult, Reader, Vec};

#[derive(Debug, Clone)]
pub struct Global {
    pub r#type: ValType,
    pub mutable: bool,
    pub init: Vec<Instruction>,
}

impl Global {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let r#type = reader.read_val_type()?;
        let mutable = reader.read_u8()? == 0x01;
        let init = Instruction::parse_vec(&mut Reader::new(reader.read_until(0x0B)))?;
        reader.read_u8()?; // consume the 0x0B
        Ok(Global {
            r#type,
            mutable,
            init,
        })
    }
}
