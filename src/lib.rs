#![no_std]
extern crate alloc;

pub mod reader;
pub mod section;
pub mod utils;

pub type Result<'a, T = ()> = core::result::Result<T, Error<'a>>;
pub use self::reader::{Mark, Reader};

use section::SectionType;
pub type Vec<T> = alloc::vec::Vec<T>;

#[derive(Debug)]
pub struct Error<'a> {
    pub mark: Mark<'a>,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    EndOfFile,
    InvalidTypeHeader,
    InvalidHeader,
    InvalidSection,
    UnknownValType,
    UnknownExportDescription,
    UnknownImportDescription,
    InvalidCode,
    UnknownRefType,

    InvalidUtf8 { inner: core::str::Utf8Error },
    IntegerOverflow(&'static str),
}

impl From<core::str::Utf8Error> for ErrorKind {
    fn from(inner: core::str::Utf8Error) -> Self {
        Self::InvalidUtf8 { inner }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Wasm<'a> {
    pub types: Vec<section::Type>,
    pub imports: Vec<section::Import<'a>>,
    pub functions: Vec<section::Function>,
    pub memories: Vec<section::Memory>,
    pub globals: Vec<section::Global>,
    pub exports: Vec<section::Export<'a>>,
    pub code: Vec<section::Code>,
    pub data: Vec<section::Data<'a>>,
    pub table: Vec<section::Table>,
}

impl<'a> Wasm<'a> {
    pub fn parse(mut reader: Reader<'a>) -> Result<'a, Self> {
        let mark = reader.mark();
        if &reader.read_exact()? != b"\0asm" {
            return Err(Error {
                mark,
                kind: ErrorKind::InvalidHeader,
            });
        }
        let mark = reader.mark();
        if reader.read_exact()? != [1, 0, 0, 0] {
            return Err(Error {
                mark,
                kind: ErrorKind::InvalidHeader,
            });
        }

        let mut types = Vec::new();
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut memories = Vec::new();
        let mut globals = Vec::new();
        let mut exports = Vec::new();
        let mut code = Vec::new();
        let mut data = Vec::new();
        let mut table = Vec::new();

        while !reader.is_empty() {
            let section_type = reader.read_section_type()?;
            let section = reader.read_slice()?;
            let mut reader = Reader::new(section);
            match section_type {
                SectionType::Type => {
                    assert!(types.is_empty());
                    types = reader.read_vec(section::Type::parse)?;
                }
                SectionType::Import => {
                    assert!(imports.is_empty());
                    imports = reader.read_vec(|r| section::Import::parse(r))?;
                }
                SectionType::Function => {
                    assert!(functions.is_empty());
                    functions = reader.read_vec(section::Function::parse)?;
                }
                SectionType::Memory => {
                    assert!(memories.is_empty());
                    memories = reader.read_vec(section::Memory::parse)?;
                }
                SectionType::Global => {
                    assert!(globals.is_empty());
                    globals = reader.read_vec(section::Global::parse)?;
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
                    assert!(data.is_empty());
                    data = reader.read_vec(section::Data::parse)?;
                }
                SectionType::Table => {
                    assert!(table.is_empty());
                    table = reader.read_vec(section::Table::parse)?;
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
            types,
            imports,
            functions,
            memories,
            globals,
            exports,
            code,
            data,
            table,
        })
    }
}
