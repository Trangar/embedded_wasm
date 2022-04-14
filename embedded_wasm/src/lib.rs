#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]
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

/// Macro that will make generating an FFI interface to a WASM file a lot easier.
///
/// # Usage
///
/// ```rs
/// struct State;
///
/// embedded_wasm::derive_ffi_handler! {
///     impl State {
///         pub fn some_function(&self) {
///             println!("Hello");
///         }
///         pub fn function_with_return_value(&self) -> i32 {
///             5
///         }
///         pub fn function_with_args(&self, counter: i32) {
///             println!("Counter is at {}", counter);
///         }
///
///         #[unhandled]
///         fn unhandled(&mut self, name: &str, args: Vec<Dynamic>) {
///             eprintln!("Unhandled method {:?} (args: {:?})", name, args);
///         }
///     }
/// }
/// ```
///
/// This will generate code like:
///
/// ```rs
/// impl embedded_wasm::FfiHandler for State {
///     fn handle(
///         &mut self,
///         process: &mut embedded_wasm::Process,
///         function_name: &str,
///         args: embedded_wasm::Vec<embedded_wasm::Dynamic>,
///     ) {
///         match function_name {
///             "some_function" => {
///                 self.some_function();
///                 return;
///                 self.unhandled(function_name, args);
///             }
///             "function_with_return_value" => {
///                 let result = self.function_with_return_value();
///                 process.stack_push(result);
///                 return;
///                 self.unhandled(function_name, args);
///             }
///             "function_with_args" => {
///                 let _0 = args.get(0).map(|a| a.as_i32());
///                 if let Some(_0) = _0 {
///                     self.function_with_args(_0);
///                     return;
///                 }
///                 self.unhandled(function_name, args);
///             }
///             _ => self.unhandled(function_name, args),
///         }
///     }
///     fn unhandled(&mut self, name: &str, args: Vec<Dynamic>) {
///         eprintln!("Unhandled method {:?} (args: {:?})", name, args);
///     }
/// }
/// impl State {
///     pub fn some_function(&self) {
///         println!("Hello");
///     }
///     pub fn function_with_return_value(&self) -> i32 {
///         5
///     }
///     pub fn function_with_args(&self, counter: i32) {
///         println!("Counter is at {}", counter);
///     }
/// }
/// ```
///
/// You can see the generated code for your specific macro invocation in `target/derive_ffi_handler.rs`
///
/// Additionally an FFI is generated that can be used from WASM. This is located in `target/embedded_wasm_ffi.rs` and looks like:
///
/// ```rs
/// extern "C" {
///     pub fn some_function();
///     pub fn function_with_return_value() -> i32;
///     pub fn function_with_args(counter: i32);
/// }
/// ```
///
/// # Limitations
///
/// Currently only the following data types are supported:
/// - `i32`
/// - `i64`
/// - `f32`
/// - `f64`
pub use embedded_wasm_derive::derive_ffi_handler;

type ParseResult<'a, T = ()> = core::result::Result<T, ParseError<'a>>;
type ExecResult<'a, T = ()> = core::result::Result<T, ExecError<'a>>;
use self::reader::{Mark, Reader};

/// The `Vec` type used in this crate.
///
/// TODO: This should be overwritable by the user somehow.
pub type Vec<T> = alloc::vec::Vec<T>;

/// Parsing error
#[derive(Debug)]
pub struct ParseError<'a> {
    /// The mark where the error occured
    pub mark: Mark<'a>,
    /// The error that occured
    pub kind: ErrorKind,
}

/// Errors that can occur while parsing.
#[derive(Debug)]
pub enum ErrorKind {
    /// The end of file was reached.
    EndOfFile,
    /// A type header was incorrect.
    InvalidTypeHeader,
    /// The header of the wasm file was incorrect.
    InvalidHeader,
    /// An invalid section type was found.
    InvalidSection,
    /// An invalid ValType was found.
    UnknownValType,
    /// An unknown export description was found.
    UnknownExportDescription,
    /// An unknown import description was found.
    UnknownImportDescription,
    /// A `code` block did not end in `0x0B`.
    InvalidCode,
    /// An invalid RefType was found.
    UnknownRefType,
    /// A 2nd `else` marker was found in the same `if` block.
    DuplicateElse,
    /// An unknown instruction was found.
    UnknownInstruction,
    /// An unknown extended instruction was found.
    UnknownExtendedInstruction,
    /// An unknown vector instruction was found.
    UnknownVectorInstruction,

    /// An invalid lane index was found.
    InvalidLaneIndex {
        /// Only lane index up to `max` is allowed
        max: u8,
    },

    /// An invalid UTF8 string is found.
    InvalidUtf8(core::str::Utf8Error),

    /// An integer overflow occured.
    IntegerOverflow(&'static str),
}

impl From<core::str::Utf8Error> for ErrorKind {
    fn from(inner: core::str::Utf8Error) -> Self {
        Self::InvalidUtf8(inner)
    }
}

#[derive(Debug)]
/// An error occured while executing a WASM file.
pub struct ExecError<'a> {
    /// The wasm file that is being executed.
    pub wasm: &'a Wasm<'a>,
    /// The error that occured.
    pub kind: ExecErrorKind,
}

/// The error description of [`ExecError`].
#[derive(Debug)]
pub enum ExecErrorKind {
    /// The given function was not found.
    FunctionNotFound,
}
