use crate::{
    instruction::{BlockType, ChildInstructions, FuncIdx, Instruction, NumType, RefType, ValType},
    ExecResult, Vec, Wasm,
};

/// A handle to a running process. This is created by using [`Wasm`]'s `spawn()` function.
pub struct Process<'a> {
    wasm: &'a Wasm<'a>,
    program_counter: Vec<ProgramCounter>,
    // memory: Vec<u8>,
    stack: Vec<Dynamic>,
}

impl<'a> Process<'a> {
    pub(crate) fn new(wasm: &'a Wasm<'a>, idx: FuncIdx) -> Self {
        Self {
            wasm,
            program_counter: {
                let mut vec = Vec::with_capacity(1);
                let code = wasm.get_code(idx);
                vec.push(ProgramCounter::new(idx, &code.locals));
                vec
            },
            // memory: {
            //     let mut vec = Vec::with_capacity(wasm.memory.0.min as usize);
            //     for _ in 0..wasm.memory.0.min {
            //         vec.push(0u8);
            //     }
            //     vec
            // },
            stack: Vec::new(),
        }
    }

    fn find_instruction<'b>(
        instructions: &'b [Instruction],
        idx: &[usize],
    ) -> Option<&'b Instruction> {
        let (idx, remaining) = idx.split_first()?;
        let instruction = instructions.get(*idx)?;

        if remaining.is_empty() {
            Some(instruction)
        } else {
            match instruction.get_child_instructions() {
                ChildInstructions::None => {
                    panic!("Expected {:?} to return child instructions", instruction)
                }
                ChildInstructions::One(children) => Self::find_instruction(children, remaining),
                ChildInstructions::Two(first, second) => {
                    let (idx, remaining) = remaining.split_first().unwrap();
                    match idx {
                        0 => Self::find_instruction(first, remaining),
                        1 => Self::find_instruction(second, remaining),
                        _ => panic!("Unknown index {:?}, expected 0 or 1", idx),
                    }
                }
            }
        }
    }

    /// Get a reference to the current instruction. For debugging purposes only, and will be removed in the future.
    pub fn current_instruction(&self) -> Instruction {
        let ProgramCounter { func, idx, .. } = self.program_counter.last().unwrap();
        let code = self.wasm.get_code(*func);
        match Self::find_instruction(&code.expr, idx) {
            Some(instruction) => instruction.clone(),
            None => panic!(
                "Could not find instruction\ncode: {:?}\nindex: {:?}",
                code.expr, idx
            ),
        }
    }

    /// Execute a single step in the wasm runner. See [`ProcessAction`] for correct handling of the return value.
    pub fn step(&mut self) -> ExecResult<ProcessAction<'a>> {
        let mut result = ProcessAction::None;
        let mut do_step = true;
        match self.current_instruction() {
            Instruction::I32Const(val) => {
                self.stack_push(val);
            }
            Instruction::Call { function } => {
                if let Some(import) = self.wasm.get_import(function) {
                    result = ProcessAction::CallExtern {
                        function: import.name.name,
                        args: core::mem::take(&mut self.stack),
                    };
                } else {
                    unimplemented!("Calling local function");
                }
            }
            Instruction::LocalSet(idx) => {
                let ProgramCounter { locals, .. } = self.program_counter.last_mut().unwrap();
                locals[idx.0] = self.stack.pop().unwrap();
            }
            Instruction::LocalGet(idx) => {
                let ProgramCounter { locals, .. } = self.program_counter.last().unwrap();
                let val = locals[idx.0].clone();
                self.stack_push(val);
            }
            Instruction::Loop { bt, inner: _ } => {
                assert_eq!(bt, BlockType::Empty);
                self.program_counter.last_mut().unwrap().idx.push(0);
                do_step = false;
            }
            Instruction::Branch { mut index } => {
                let ProgramCounter { idx, func, .. } = self.program_counter.last_mut().unwrap();
                let code = self.wasm.get_code(*func);
                loop {
                    idx.pop();
                    if index.0 > 0 {
                        index.0 -= 1;
                    } else {
                        break;
                    }
                }
                match Self::find_instruction(&code.expr, idx).unwrap().clone() {
                    Instruction::Loop { bt, .. } => {
                        assert_eq!(bt, BlockType::Empty);
                        idx.push(0);
                        do_step = false;
                    }
                    x if x.get_child_instructions().is_none() => {}
                    x => unimplemented!("Loop handling on {:?}", x),
                }
            }
            x => unimplemented!("Unimplemented instruction: {:?}", x),
        }
        if do_step {
            *self
                .program_counter
                .last_mut()
                .unwrap()
                .idx
                .last_mut()
                .unwrap() += 1;
        }
        Ok(result)
    }

    /// Push a value onto the stack. This should only be called when [`ProcessAction`] `CallExten` is returned from `step`
    pub fn stack_push(&mut self, val: impl Into<Dynamic>) {
        self.stack.push(val.into());
    }
}

/// Result of [`Process`]' `step` function.
pub enum ProcessAction<'a> {
    /// No action should be taken, you can freely call `step` again.
    None,
    /// The function finished with the given return values.
    Finished(Vec<Dynamic>),
    /// The wasm binary tried calling the given function. You *have* to do one of two things:
    ///
    /// - If you're using [`derive_ffi_handler`], call `embedded_wasm::FfiHandler::handle(&mut your_state, &mut process, function, args)`
    /// - If you're handling this manually, make sure to call `process::stack_push` for each return value that your function returns.
    ///
    /// [`derive_ffi_handler`]: macro.derive_ffi_handler.html
    CallExtern {
        /// The function name that is being called.
        function: &'a str,
        /// The arguments of the function that were passed from the code.
        args: Vec<Dynamic>,
    },
}

/// A dynamic value type.
#[derive(Debug, Clone)]
pub struct Dynamic([u8; 8]);

impl From<i32> for Dynamic {
    fn from(i: i32) -> Self {
        let mut bytes = [0u8; 8];
        bytes[..4].copy_from_slice(&i.to_le_bytes());
        Self(bytes)
    }
}

impl From<i64> for Dynamic {
    fn from(i: i64) -> Self {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&i.to_le_bytes());
        Self(bytes)
    }
}

impl From<f32> for Dynamic {
    fn from(i: f32) -> Self {
        let mut bytes = [0u8; 8];
        bytes[..4].copy_from_slice(&i.to_le_bytes());
        Self(bytes)
    }
}

impl From<f64> for Dynamic {
    fn from(i: f64) -> Self {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&i.to_le_bytes());
        Self(bytes)
    }
}

impl Dynamic {
    /// Cast the dynamic value to an `i32`
    pub fn as_i32(&self) -> i32 {
        i32::from_le_bytes(self.0[..4].try_into().unwrap())
    }
    /// Cast the dynamic value to an `f32`
    pub fn as_f32(&self) -> f32 {
        f32::from_le_bytes(self.0[..4].try_into().unwrap())
    }
    /// Cast the dynamic value to an `i64`
    pub fn as_i64(&self) -> i64 {
        i64::from_le_bytes(self.0)
    }
    /// Cast the dynamic value to an `f64`
    pub fn as_f64(&self) -> f64 {
        f64::from_le_bytes(self.0)
    }
}

impl From<ValType> for Dynamic {
    fn from(t: ValType) -> Self {
        match t {
            ValType::Num(num_type) => num_type.into(),
            ValType::Ref(ref_type) => ref_type.into(),
        }
    }
}

impl From<NumType> for Dynamic {
    fn from(n: NumType) -> Self {
        match n {
            NumType::I32 | NumType::I64 => Self([0u8; 8]),
            NumType::F32 => 0.0f32.into(),
            NumType::F64 => 0.0f64.into(),
        }
    }
}

impl From<RefType> for Dynamic {
    fn from(r: RefType) -> Self {
        match r {
            RefType::FuncRef => todo!(),
            RefType::ExternRef => todo!(),
        }
    }
}

struct ProgramCounter {
    pub func: FuncIdx,
    pub idx: Vec<usize>,
    pub locals: Vec<Dynamic>,
}

impl ProgramCounter {
    #[allow(clippy::vec_init_then_push)]
    pub fn new(func: FuncIdx, l: &[(u32, ValType)]) -> Self {
        let mut locals = Vec::with_capacity(l.iter().map(|(count, _)| *count as usize).sum());
        for (count, local) in l.iter().copied() {
            for _ in 0..count {
                locals.push(local.into())
            }
        }
        let mut idx = Vec::new();
        idx.push(0);
        Self { func, idx, locals }
    }
}
