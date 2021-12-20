use super::TypeIdx;
use crate::{Reader, Result};

#[derive(Debug, Clone)]
pub struct Function(pub TypeIdx);

impl Function {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        Ok(Self(reader.read_index()?))
    }
}
