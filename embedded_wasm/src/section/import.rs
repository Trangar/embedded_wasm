use super::{GlobalIdx, MemIdx, TableIdx, TypeIdx};
use crate::{ErrorKind, ParseResult, Reader};

#[derive(Clone, Debug)]
pub struct Import<'a> {
    pub name: NamespaceName<'a>,
    pub desc: ImportDescription,
}

#[derive(Clone, Debug)]
pub struct NamespaceName<'a> {
    pub namespace: &'a str,
    pub name: &'a str,
}

impl<'a> Import<'a> {
    pub fn parse(reader: &mut Reader<'a>) -> ParseResult<'a, Self> {
        let namespace = reader.read_str()?;
        let name = reader.read_str()?;
        let mark = reader.mark();
        let desc = match reader.read_u8()? {
            0x00 => ImportDescription::Type(reader.read_index()?),
            0x01 => ImportDescription::Table(reader.read_index()?),
            0x02 => ImportDescription::Memory(reader.read_index()?),
            0x03 => ImportDescription::Global(reader.read_index()?),
            _ => return Err(mark.to_error(ErrorKind::UnknownImportDescription)),
        };
        Ok(Self {
            name: NamespaceName { namespace, name },
            desc,
        })
    }
}

#[derive(Clone, Debug)]
pub enum ImportDescription {
    Type(TypeIdx),
    Table(TableIdx),
    Memory(MemIdx),
    Global(GlobalIdx),
}
