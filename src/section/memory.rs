use crate::{Reader, Result};

use super::Limit;

#[derive(Clone, Debug)]
pub struct Memory(pub Limit);

impl Memory {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> Result<'a, Self> {
        Ok(Self(Limit::parse(reader)?))
    }
}
