use crate::{process::Dynamic, Process, Vec};

/// A handler that can be used to handle function calls from [`ProcessAction`].
///
/// This is mostly used in combination with [`derive_ffi_handler`].
///
/// [`ProcessAction`]: enum.ProcessAction.html
/// [`derive_ffi_handler`]: macro.derive_ffi_handler.html
pub trait FfiHandler {
    /// Handle an given `function` with `args` arguments.
    ///
    /// Implementors of this function should make sure that all the return values are stored in `process.push_stack` in the correct order.
    fn handle(&mut self, process: &mut Process, function: &str, args: Vec<Dynamic>);

    /// Function that can be used to report that a function call was unhandled. This will be called if:
    /// - A function doesn't exist.
    /// - A function's arguments don't match.
    fn unhandled(&mut self, _function: &str, _args: Vec<Dynamic>) {}
}
