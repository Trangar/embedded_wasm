use crate::{
    instruction::{FuncIdx, GlobalIdx, MemIdx, TableIdx},
    ErrorKind, ParseResult, Reader,
};

#[derive(Clone, Debug)]
pub struct Export<'a> {
    pub name: &'a str,
    pub desc: ExportDesc,
}

impl<'a> Export<'a> {
    pub fn parse(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let name = reader.read_str()?;
        let mark = reader.mark();
        let desc = match reader.read_u8()? {
            0x00 => ExportDesc::Function(reader.read_index()?),
            0x01 => ExportDesc::Table(reader.read_index()?),
            0x02 => ExportDesc::Memory(reader.read_index()?),
            0x03 => ExportDesc::Global(reader.read_index()?),
            _ => return Err(mark.into_error(ErrorKind::UnknownExportDescription)),
        };
        Ok(Export { name, desc })
    }
}

#[derive(Clone, Debug)]
pub enum ExportDesc {
    Function(FuncIdx),
    Table(TableIdx),
    Memory(MemIdx),
    Global(GlobalIdx),
}
