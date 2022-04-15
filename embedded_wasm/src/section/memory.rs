use super::Limit;
use crate::{ParseResult, Reader};

#[derive(Clone, Debug)]
pub struct Memory(pub Limit);

impl Memory {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Ok(Self(Limit::parse(reader)?))
    }
}
