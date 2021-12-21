use super::{Limit, RefType};
use crate::{Reader, Result};

#[derive(Debug, Clone)]
pub struct Table {
    pub reftype: RefType,
    pub limits: Limit,
}

impl Table {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        let reftype = {
            let mark = reader.mark();
            let val = reader.read_u8()?;
            RefType::from_u8(val).map_err(|e| mark.to_error(e))?
        };
        let limits = Limit::parse(reader)?;
        Ok(Self { reftype, limits })
    }
}
