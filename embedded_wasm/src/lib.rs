#![no_std]
extern crate alloc;

mod ffi;
mod instruction;
mod process;
mod reader;
mod section;
mod utils;
mod wasm;

pub use self::{
    ffi::FfiHandler,
    process::{Dynamic, Process, ProcessAction},
    wasm::Wasm,
};
pub use embedded_wasm_derive::derive_ffi_handler;

type ParseResult<'a, T = ()> = core::result::Result<T, ParseError<'a>>;
type ExecResult<'a, T = ()> = core::result::Result<T, ExecError<'a>>;
use self::reader::{Mark, Reader};

pub type Vec<T> = alloc::vec::Vec<T>;

#[derive(Debug)]
pub struct ParseError<'a> {
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
    DuplicateElse,
    UnknownInstruction,
    UnknownExtendedInstruction,
    UnknownVectorInstruction,

    InvalidLaneIndex { max: u8 },

    InvalidUtf8 { inner: core::str::Utf8Error },
    IntegerOverflow(&'static str),
}

impl From<core::str::Utf8Error> for ErrorKind {
    fn from(inner: core::str::Utf8Error) -> Self {
        Self::InvalidUtf8 { inner }
    }
}

#[derive(Debug)]
pub struct ExecError<'a> {
    pub wasm: &'a Wasm<'a>,
    pub kind: ExecErrorKind,
}

#[derive(Debug)]
pub enum ExecErrorKind {
    FunctionNotFound,
}
