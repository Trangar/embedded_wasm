use crate::{
    instruction::{FuncIdx, SectionType},
    reader::Reader,
    section, ErrorKind, ExecError, ExecErrorKind, ExecResult, ParseError, ParseResult, Process,
    Vec,
};

/// A reference to a parsed WASM file.
///
/// This is created by calling `parse`, and can be executed by calling `spawn`.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Wasm<'a> {
    // types: Vec<section::Type>,
    imports: Vec<section::Import<'a>>,
    // functions: Vec<section::Function>,
    /// A reference to the memory segment in this wasm file
    #[cfg(feature = "parse-memory")]
    #[cfg_attr(docsrs, doc(cfg(feature = "parse-memory")))]
    pub memory: section::Memory,

    /// A reference to the globals in this wasm file
    #[cfg(feature = "parse-globals")]
    #[cfg_attr(docsrs, doc(cfg(feature = "parse-globals")))]
    pub globals: Vec<section::Global>,

    exports: Vec<section::Export<'a>>,
    code: Vec<section::Code>,
    // data: Vec<section::Data<'a>>,
    // table: Vec<section::Table>,
}

impl<'a> Wasm<'a> {
    /// Parse a wasm file.
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
        #[cfg(feature = "parse-memory")]
        let mut memory = None;
        #[cfg(feature = "parse-globals")]
        let mut globals = Vec::new();
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
                    #[cfg(feature = "parse-memory")]
                    {
                        assert!(memory.is_none());
                        let mut memories = reader.read_vec(section::Memory::parse)?;
                        assert!(memories.len() == 1);
                        memory = Some(memories.remove(0));
                    }
                }
                SectionType::Global => {
                    #[cfg(feature = "parse-globals")]
                    {
                        assert!(globals.is_empty());
                        globals = reader.read_vec(section::Global::parse)?;
                    }
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
            #[cfg(feature = "parse-memory")]
            memory: memory.expect("Missing memory block"),
            #[cfg(feature = "parse-globals")]
            globals,
            exports,
            code,
            // data,
            // table,
        })
    }

    /// Spawn a new process that starts at the given `fn_name` entrypoint.
    ///
    /// Implementations should make sure that the given `fn_name` is publicly available. In rust this is done by marking it as:
    /// ```rs
    /// #[no_mangle]
    /// pub extern "C" fn fn_name() { .. }
    /// ```
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

    pub(crate) fn get_code(&self, idx: FuncIdx) -> &section::Code {
        &self.code[idx.0 - self.imports.len()]
    }

    pub(crate) fn get_import(&self, idx: FuncIdx) -> Option<&section::Import> {
        self.imports.get(idx.0)
    }
}
