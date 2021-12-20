use crate::{Reader, Result};

#[derive(Clone, Debug)]
pub struct Import<'a> {
    pub name: NamespaceName<'a>,
}

#[derive(Clone, Debug)]
pub struct NamespaceName<'a> {
    pub namespace: &'a str,
    pub name: &'a str,
}

impl<'a> Import<'a> {
    pub fn parse(reader: &mut Reader<'a>) -> Result<'a, Self> {
        let namespace = reader.read_str()?;
        let name = reader.read_str()?;
        Ok(Self {
            name: NamespaceName { namespace, name },
        })
    }
}
