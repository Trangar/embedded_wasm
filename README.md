# Embedded WASM

A toy WASM interpreter, designed to run on microcontrollers with minimal resources available.

## Setting up a new project

1. Create the following projects:
   1. `shared` for sharing types between the runner and the wasm binary (e.g. for enums)
   1. `sys` for the FFI interface. This should have a safe wrapper around several `extern "C"` functions.
   1. 1 or more crate that get compiled to `wasm32-unknown-unknown`, using the `shared` and `sys` crates.
   1. A `runner` crate that runs your WASM interpreter. This references the `shared` crate.
1. Create a FFI
   1. In your `runner` crate, make sure you can parse the FFI calls (`ProcessAction::CallExtern { function, args }`)
   1. Add the function to your `sys` crate
1. Run your WASM
   1. Parse it with `Wasm::parse`
   1. Spawn a new process with `wasm.spawn("start_fn_name")`
   1. Continuously poll `process.step()` and handle the `ProcessAction` result

See `runner/desktop` and `projects/sys` for an example.

## Building this project

You need the following tools:

- Rust nightly (any nightly will do)
  - Target `wasm32-unknown-unknown`
- For rp2040:
  - target `thumbv6m-none-eabi`
  - `cargo install elf2uf2-rs --locked`
  - For debugging: `cargo install cargo-embed --locked`

First build `projects/blink`. This will produce a wasm in the `projects/blink/target/wasm32-unknown-unknown` folder. (For microcontrollers, `cargo build --release` is recommended)

Then build one of:
- Desktop runner: `cargo run -- ./projects/blink/target/wasm32-unknown-unknown/release/blink.wasm`
- RP2040: 
  - `./flash.sh` or `./flash.ps1` for flashing in DAP mode (mounted as a file system)
  - `cargo embed` for flashing and debugging through a SWD debugger

## Project layout

### Embedded_wasm

The crate that parses wasm and provides handles to processes.

### Projects

Projects are applications compiled to wasm. This folder is structured as followed:
- `blink` is a simple blink script.
- `shared` is a shared library. This is used to have the same type definitions between `projects/blink` and any of the runners.
- `sys` is the safe FFI interface for the scripts.

### Runners
Runners are binaries that run on certain platforms, and run a given binary.

There are currently 2 runners:
- `desktop`: Runs on a pc as a CLI tool. Takes the WASM file as the first arg.
- `rp2040`: Runs a WASM file on a [rp2040](https://www.raspberrypi.com/products/rp2040/) ([hal](https://docs.rs/rp2040-hal)) microcontroller. The WASM file is currently embedded through `include_bytes!()`

## Useful tools

[https://webassembly.github.io/wabt/demo/wasm2wat/](https://webassembly.github.io/wabt/demo/wasm2wat/)

Upload a .wasm here to inspect the contents
