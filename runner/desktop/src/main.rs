use clap::Parser;
use embedded_wasm::{Dynamic, ProcessAction, Wasm};
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

    let mut state = State;

    loop {
        // dbg!(process.current_instruction());
        match process.step().unwrap() {
            ProcessAction::None => {}
            ProcessAction::Result(res) => {
                panic!("Start function exited with value {:?}", res);
            }
            ProcessAction::CallExtern { function, args } => {
                embedded_wasm::FfiHandler::handle(&mut state, &mut process, function, args)
            }
        }
    }
}

struct State;

embedded_wasm::derive_ffi_handler! {
    impl State {
        pub fn get_led_handle(&self, idx: i32) -> i32 {
            idx
        }

        pub fn led_on(&self, idx: i32) {
            println!("Led {:?} on!", idx);
        }

        pub fn led_off(&self, idx: i32) {
            println!("Led {:?} off!", idx);
        }

        pub fn delay(&self, sleep_ms: i32) {
            println!("Sleeping for {:?} ms", sleep_ms);
            std::thread::sleep(Duration::from_millis(sleep_ms as _));
        }

        #[unhandled]
        fn unhandled(&mut self, name: &str, args: Vec<Dynamic>) {
            eprintln!("Unhandled method {:?} (args: {:?})", name, args);
        }
    }
}
