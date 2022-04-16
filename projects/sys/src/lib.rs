#![no_std]
#![feature(waker_getters, never_type, alloc_error_handler)]

pub extern crate embedded_time;
pub mod future;

extern crate alloc;

mod allocator;

use core::ops;
use embedded_time::duration::Milliseconds;
use shared::LedIndex;

pub(crate) mod ffi {
    extern "C" {
        pub fn get_led_handle(led: i32) -> i32;
        pub fn led_on(led_handle: i32);
        pub fn led_off(led_handle: i32);
        pub fn delay(sleep_ms: i32);
        pub fn get_time_millis() -> i32;
        pub fn notify_after(ms: i32, task: i32);
        pub fn wait_for_interrupt();
        pub fn disable_interrupts();
        pub fn enable_interrupts();

        pub fn alloc(size: usize, align: usize) -> *mut u8;
        pub fn dealloc(ptr: *mut u8, size: usize, align: usize);
    }
}

pub fn interrupt_free<F: FnOnce() -> T, T>(f: F) -> T {
    unsafe {
        ffi::disable_interrupts();
    }

    // Whenever this struct goes out of scope, it will re-enable interrupts.
    // This construction is needed because `f` might panic.
    // When rust is panicing, all `Drop` implementations still get called.
    struct ReenableInterruptOnDrop;
    impl Drop for ReenableInterruptOnDrop {
        fn drop(&mut self) {
            unsafe { ffi::enable_interrupts() }
        }
    }

    let _reenable_on_drop = ReenableInterruptOnDrop;
    f()
    // `enable_interrupt` is automatically called when `_reenable_on_drop` goes out of scope
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Instant(i32);

impl Instant {
    pub fn now() -> Self {
        Self(unsafe { ffi::get_time_millis() })
    }

    pub fn elapsed_millis(&self) -> Milliseconds {
        Instant::now() - self.clone()
    }
}

impl ops::Add<Milliseconds> for Instant {
    type Output = Instant;
    fn add(self, other: Milliseconds) -> Instant {
        Self(self.0 + other.0 as i32)
    }
}

impl ops::Sub for Instant {
    type Output = Milliseconds;
    fn sub(self, other: Instant) -> Milliseconds {
        let result = other.0.saturating_sub(self.0);
        if result < 0 {
            Milliseconds(0)
        } else {
            Milliseconds(result as u32)
        }
    }
}

pub fn delay(time: impl Into<Milliseconds>) {
    unsafe {
        ffi::delay(time.into().0 as i32);
    }
}

#[cfg(target_family = "wasm")]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub struct Led {
    handle: i32,
}

impl Led {
    fn get(idx: LedIndex) -> Self {
        let handle = unsafe { ffi::get_led_handle(idx as i32) };
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
