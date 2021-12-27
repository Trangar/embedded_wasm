use crate::{
    reader::Reader, section, section::SectionType, ErrorKind, ExecError, ExecErrorKind, ExecResult,
    ParseError, ParseResult, Process, Vec,
};

#[derive(Debug)]
pub struct Wasm<'a> {
    // types: Vec<section::Type>,
    imports: Vec<section::Import<'a>>,
    // functions: Vec<section::Function>,
    // memory: section::Memory,
    // globals: Vec<section::Global>,
    exports: Vec<section::Export<'a>>,
    code: Vec<section::Code>,
    // data: Vec<section::Data<'a>>,
    // table: Vec<section::Table>,
}

impl<'a> Wasm<'a> {
    pub fn parse(slice: &'a [u8]) -> ParseResult<'a, Self> {
        let mut reader = Reader::new(slice);
        let mark = reader.mark();
        if &reader.read_exact()? != b"\0asm" {
            return Err(ParseError {
                mark,
                kind: ErrorKind::InvalidHeader,
            });
        }
        let mark = reader.mark();
        if reader.read_exact()? != [1, 0, 0, 0] {
            return Err(ParseError {
                mark,
                kind: ErrorKind::InvalidHeader,
            });
        }

        // let mut types = Vec::new();
        let mut imports = Vec::new();
        // let mut functions = Vec::new();
        // let mut memory = None;
        // let mut globals = Vec::new();
        let mut exports = Vec::new();
        let mut code = Vec::new();
        // let mut data = Vec::new();
        // let mut table = Vec::new();

        while !reader.is_empty() {
            let section_type = SectionType::parse(&mut reader)?;
            let section = reader.read_slice()?;
            let mut reader = Reader::new(section);
            match section_type {
                SectionType::Type => {
                    // assert!(types.is_empty());
                    // types = reader.read_vec(section::Type::parse)?;
                }
                SectionType::Import => {
                    assert!(imports.is_empty());
                    imports = reader.read_vec(|r| section::Import::parse(r))?;
                }
                SectionType::Function => {
                    // assert!(functions.is_empty());
                    // functions = reader.read_vec(section::Function::parse)?;
                }
                SectionType::Memory => {
                    // assert!(memory.is_none());
                    // let mut memories = reader.read_vec(section::Memory::parse)?;
                    // assert!(memories.len() == 1);
                    // memory = Some(memories.remove(0));
                }
                SectionType::Global => {
                    // assert!(globals.is_empty());
                    // globals = reader.read_vec(section::Global::parse)?;
                }
                SectionType::Export => {
                    assert!(exports.is_empty());
                    exports = reader.read_vec(section::Export::parse)?;
                }
                SectionType::Code => {
                    assert!(code.is_empty());
                    code = reader.read_vec(section::Code::parse)?;
                }
                SectionType::Data => {
                    // assert!(data.is_empty());
                    // data = reader.read_vec(section::Data::parse)?;
                }
                SectionType::Table => {
                    // assert!(table.is_empty());
                    // table = reader.read_vec(section::Table::parse)?;
                }
                SectionType::Custom => {
                    // ignored
                }
                x => panic!(
                    "WARNING: Section type {:?}\n  {:02x?}\n  {}",
                    x,
                    reader.remaining(),
                    alloc::string::String::from_utf8_lossy(reader.remaining())
                ),
            }
        }
        Ok(Self {
            // types,
            imports,
            // functions,
            // memory: memory.expect("Missing memory block"),
            // globals,
            exports,
            code,
            // data,
            // table,
        })
    }

    pub fn spawn(&'a self, fn_name: &str) -> ExecResult<'a, Process<'a>> {
        let entry_func_idx = match self
            .exports
            .iter()
            .filter_map(|e| match &e.desc {
                section::ExportDesc::Function(idx) => Some((idx, e.name)),
                _ => None,
            })
            .find(|(_, name)| *name == fn_name)
        {
            Some((idx, _)) => idx,
            None => {
                return Err(ExecError {
                    wasm: self,
                    kind: ExecErrorKind::FunctionNotFound,
                })
            }
        };

        Ok(Process::new(self, *entry_func_idx))
    }

    pub fn get_code(&self, idx: section::FuncIdx) -> &section::Code {
        &self.code[idx.0 - self.imports.len()]
    }

    pub fn get_import(&self, idx: section::FuncIdx) -> Option<&section::Import> {
        self.imports.get(idx.0)
    }
}
