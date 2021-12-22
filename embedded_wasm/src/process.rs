use crate::{
    section::{BlockType, ChildInstructions, FuncIdx, Instruction, NumType, RefType, ValType},
    ExecResult, Vec, Wasm,
};

pub struct Process<'a> {
    wasm: &'a Wasm<'a>,
    program_counter: Vec<ProgramCounter>,
    // memory: Vec<u8>,
    stack: Vec<Dynamic>,
}

impl<'a> Process<'a> {
    pub fn new(wasm: &'a Wasm<'a>, idx: FuncIdx) -> Self {
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

    pub fn current_instruction(&self) -> Instruction {
        let ProgramCounter { func, idx, .. } = self.program_counter.last().unwrap();
        let code = &self.wasm.code[func.0 - self.wasm.imports.len()];
        match Self::find_instruction(&code.expr, idx) {
            Some(instruction) => instruction.clone(),
            None => panic!(
                "Could not find instruction\ncode: {:?}\nindex: {:?}",
                code.expr, idx
            ),
        }
    }

    pub fn step(&mut self) -> ExecResult<ProcessAction<'a>> {
        let mut result = ProcessAction::None;
        let mut do_step = true;
        match self.current_instruction() {
            Instruction::I32Const(val) => {
                self.stack_push(val);
            }
            Instruction::Call { function } => {
                if function.0 < self.wasm.imports.len() {
                    result = ProcessAction::CallExtern {
                        function: self.wasm.imports[function.0].name.name,
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

    pub fn stack_push(&mut self, val: impl Into<Dynamic>) {
        self.stack.push(val.into());
    }
}

pub enum ProcessAction<'a> {
    None,
    Result(Vec<Dynamic>),
    CallExtern {
        function: &'a str,
        args: Vec<Dynamic>,
    },
}

#[derive(Debug, Clone)]
pub enum Dynamic {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl From<i32> for Dynamic {
    fn from(i: i32) -> Self {
        Self::I32(i)
    }
}

impl Dynamic {
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Dynamic::I32(val) => Some(*val),
            // TODO: Cast other values to u32?
            _ => None,
        }
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
            NumType::I32 => Self::I32(0),
            NumType::I64 => Self::I64(0),
            NumType::F32 => Self::F32(0.0),
            NumType::F64 => Self::F64(0.0),
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
