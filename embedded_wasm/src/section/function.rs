use super::TypeIdx;
use crate::{ParseResult, Reader};

#[derive(Debug, Clone)]
pub struct Function(pub TypeIdx);

impl Function {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        Ok(Self(reader.read_index()?))
    }
}
