use super::{Instruction, MemIdx};
use crate::{Reader, Result};
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Data<'a> {
    pub memidx: Option<MemIdx>,
    pub expression: Option<Vec<Instruction>>,
    pub bytes: &'a [u8],
}

impl<'a> Data<'a> {
    pub fn parse(reader: &mut Reader<'a>) -> Result<'a, Self> {
        let mode = reader.read_u8()?;
        let memidx = if mode == 0x02 {
            Some(reader.read_index()?)
        } else {
            None
        };
        let expression = if mode == 0x00 || mode == 0x02 {
            let bytes = reader.read_until(0x0b);
            reader.read_u8()?; // read 0x0b

            let mut reader = Reader::new(bytes);
            Some(Instruction::parse_vec(&mut reader)?)
        } else {
            None
        };
        let bytes = reader.read_slice()?;
        Ok(Self {
            memidx,
            expression,
            bytes,
        })
    }
}
