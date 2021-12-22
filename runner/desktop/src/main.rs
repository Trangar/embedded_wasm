use clap::Parser;
use embedded_wasm::{process::ProcessAction, Wasm};
use num_traits::FromPrimitive;
use std::{fs::File, io::Write, path::PathBuf, time::Duration};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Location of the .wasm file
    wasm: PathBuf,
}

fn main() {
    let args = Args::parse();
    let bytes = std::fs::read(args.wasm).unwrap();
    // include_bytes!("../main.wasm");
    let wasm = Wasm::parse(&bytes).unwrap();
    let mut fs = File::create("out.txt").unwrap();
    write!(fs, "{:#?}", wasm).unwrap();
    drop(fs);

    let mut process = wasm.spawn("start").unwrap();

    loop {
        // dbg!(process.current_instruction());
        match process.step().unwrap() {
            ProcessAction::None => {}
            ProcessAction::Result(res) => {
                panic!("Start function exited with value {:?}", res);
            }
            ProcessAction::CallExtern { function, args } => match function {
                "get_led_handle" => {
                    if let Some(idx) = args
                        .first()
                        .and_then(|a| a.as_i32())
                        .and_then(shared::LedIndex::from_i32)
                    {
                        process.stack_push(idx as i32);
                    } else {
                        panic!("Failed to call {:?}, unknown args: {:?}", function, args);
                    }
                }
                "led_on" => {
                    if let Some(idx) = args
                        .first()
                        .and_then(|a| a.as_i32())
                        .and_then(shared::LedIndex::from_i32)
                    {
                        println!("Led {:?} on!", idx);
                    } else {
                        panic!("Failed to call {:?}, unknown args: {:?}", function, args);
                    }
                }
                "led_off" => {
                    if let Some(idx) = args
                        .first()
                        .and_then(|a| a.as_i32())
                        .and_then(shared::LedIndex::from_i32)
                    {
                        println!("Led {:?} off!", idx);
                    } else {
                        panic!("Failed to call {:?}, unknown args: {:?}", function, args);
                    }
                }
                "delay" => {
                    if let Some(sleep_ms) = args.first().and_then(|a| a.as_i32()) {
                        println!("Sleeping for {:?} ms", sleep_ms);
                        std::thread::sleep(Duration::from_millis(sleep_ms as _));
                    } else {
                        panic!("Failed to call {:?}, unknown args: {:?}", function, args);
                    }
                }
                x => panic!("Unknown extern function {:?} (args: {:?})", x, args),
            },
        }
    }
}
