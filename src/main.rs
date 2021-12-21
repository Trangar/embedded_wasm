extern crate embedded_wasm;

use embedded_wasm::{section, Reader, Wasm};
use std::{fs::File, io::Write};

fn main() {
    let bytes =
        include_bytes!("../projects/empty/target/wasm32-unknown-unknown/release/empty.wasm");
    // include_bytes!("../main.wasm");
    let wasm = Wasm::parse(Reader::new(bytes)).unwrap();
    let mut fs = File::create("out.txt").unwrap();
    write!(fs, "{:#?}", wasm).unwrap();
    drop(fs);
    let entry_func_idx = match wasm
        .exports
        .iter()
        .filter_map(|e| match &e.desc {
            section::ExportDesc::Function(idx) => Some((idx, e.name)),
            _ => None,
        })
        .find(|(_, name)| *name == "start")
    {
        Some((idx, _)) => idx,
        None => panic!("Could not find entry point \"start\""),
    };
    let code_idx = entry_func_idx.0 - wasm.imports.len();
    let function = &wasm.functions[code_idx];
    let ty = &wasm.types[(function.0).0];
    println!(
        "Found function \"start\" (fn ({:?}) -> {:?})",
        ty.lhs, ty.rhs
    );
    let code = &wasm.code[code_idx];

    println!("Globals:");
    for (idx, global) in wasm.globals.iter().enumerate() {
        println!("  {}: {:?}", idx, global);
    }
    println!("Locals:");
    let mut idx = 0;
    for (count, r#type) in &code.locals {
        for _ in 0..*count {
            println!("  {}: {:?}", idx, r#type);
            idx += 1;
        }
    }
    println!("Instructions:");

    fn print_instruction_list(wasm: &Wasm, instructions: &[section::Instruction], depth: usize) {
        let prefix = " ".repeat(depth);
        for instruction in instructions {
            match instruction {
                section::Instruction::Loop { bt, inner } => {
                    println!("{}Loop (BlockType {:?})", prefix, bt);
                    print_instruction_list(wasm, inner, depth + 2);
                }
                section::Instruction::Call { function } => {
                    print!("{}Call {:?}", prefix, function);
                    if let Some(import) = wasm.imports.get(function.0) {
                        println!(": {}", import.name.name);
                    } else {
                        let code = &wasm.code[function.0 - wasm.imports.len()];

                        println!(" (Custom function, has {} locals)", code.locals.len());
                        if !code.locals.is_empty() {
                            let mut idx = 0;
                            for (count, r#type) in &code.locals {
                                for _ in 0..*count {
                                    println!("{}    {}: {:?}", prefix, idx, r#type);
                                    idx += 1;
                                }
                            }
                            println!();
                        }
                        print_instruction_list(wasm, &code.expr, depth + 4);
                    }
                }
                x => println!("{}{:?}", prefix, x),
            }
        }
    }
    print_instruction_list(&wasm, &code.expr, 2);
}
