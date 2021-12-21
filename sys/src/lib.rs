#![no_std]

pub extern crate embedded_time;

use embedded_time::duration::Milliseconds;

#[repr(C)]
pub struct Test {
    a: u64,
    b: u64,
    c: u64,
}
mod ffi {
    extern "C" {
        pub fn get_led_handle(led: u32) -> u32;
        pub fn led_on(led_handle: u32);
        pub fn led_off(led_handle: u32);
        pub fn delay(ms: u32);
    }
}

pub fn delay(time: impl Into<Milliseconds>) {
    unsafe {
        ffi::delay(time.into().0);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub struct Led {
    handle: u32,
}

impl Led {
    pub fn get(idx: LedIndex) -> Self {
        let handle = unsafe { ffi::get_led_handle(idx as u32) };
        Self { handle }
    }

    pub fn a0() -> Self {
        Self::get(LedIndex::A0)
    }

    pub fn on(&mut self) {
        unsafe { ffi::led_on(self.handle) };
    }

    pub fn off(&mut self) {
        unsafe { ffi::led_off(self.handle) };
    }
}

pub enum LedIndex {
    Error = 1,
    Step,
    Run,
    D0,
    D1,
    D2,
    D3,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    IO0,
    IO1,
    IO2,
    IO3,
    IO4,
    IO5,
    IO6,
    IO7,
}
