#![no_std]

#[repr(C)]
pub struct Test {
    a: u64,
    b: u64,
    c: u64,
}
mod ffi {
    extern "C" {
        pub fn alert(s: *const u8, len: usize) -> crate::Test;
    }
}

pub fn alert(s: &str) {
    unsafe {
        ffi::alert(s.as_ptr(), s.len());
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
