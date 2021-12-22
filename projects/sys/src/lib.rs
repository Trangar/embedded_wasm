#![no_std]

pub extern crate embedded_time;
use shared::LedIndex;

use embedded_time::duration::Milliseconds;

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

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub struct Led {
    handle: u32,
}

impl Led {
    fn get(idx: LedIndex) -> Self {
        let handle = unsafe { ffi::get_led_handle(idx as u32) };
        Self { handle }
    }

    pub fn d1() -> Self {
        Self::get(LedIndex::D1)
    }

    pub fn d2() -> Self {
        Self::get(LedIndex::D2)
    }

    pub fn d3() -> Self {
        Self::get(LedIndex::D3)
    }

    pub fn d4() -> Self {
        Self::get(LedIndex::D4)
    }

    pub fn d5() -> Self {
        Self::get(LedIndex::D5)
    }

    pub fn on(&mut self) {
        unsafe { ffi::led_on(self.handle) };
    }

    pub fn off(&mut self) {
        unsafe { ffi::led_off(self.handle) };
    }
}
