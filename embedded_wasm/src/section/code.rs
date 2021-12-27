use super::ValType;
use crate::{instruction::Instruction, ErrorKind, ParseResult, Reader, Vec};

#[derive(Debug)]
pub struct Code {
    pub locals: Vec<(u32, ValType)>,
    pub expr: Vec<Instruction>,
}

impl Code {
    pub fn parse<'a>(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let slice = reader.read_slice()?;
        let (last_byte, slice) = slice.split_last().unwrap();
        if *last_byte != 0x0B {
            let mark = reader.mark_relative(-1);
            return Err(mark.to_error(ErrorKind::InvalidCode));
        }
        let mut reader = Reader::new(slice);

        // TODO: Flatten this into a Vec<ValType>?
        let locals = reader.read_vec(|reader| {
            let count = reader.read_int()?;
            let val_type = ValType::parse(reader)?;
            Ok((count, val_type))
        })?;

        let expr = Instruction::parse_vec(&mut reader)?;

        Ok(Self { locals, expr })
    }
}
