#![no_std]

#[no_mangle]
pub extern "C" fn main() {
    sys::alert("Hello world!");
}
