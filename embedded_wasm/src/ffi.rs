use crate::{process::Dynamic, Process, Vec};

pub trait FfiHandler {
    fn handle(&mut self, process: &mut Process, function: &str, args: Vec<Dynamic>);
    fn unhandled(&mut self, _function: &str, _args: Vec<Dynamic>) {}
}