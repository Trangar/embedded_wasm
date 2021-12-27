use super::{Limit, RefType};
use crate::{ParseResult, Reader};

#[derive(Debug, Clone)]
pub struct Table {
    pub reftype: RefType,
    pub limits: Limit,
}

impl Table {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let reftype = RefType::parse(reader)?;
        let limits = Limit::parse(reader)?;
        Ok(Self { reftype, limits })
    }
}
