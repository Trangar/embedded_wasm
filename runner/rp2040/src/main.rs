#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc_cortex_m::CortexMHeap;
use core::convert::Infallible;
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::rate::*;
use embedded_wasm::{ProcessAction, Vec, Wasm};
use num_traits::FromPrimitive;
use rp2040_hal::{
    clocks::init_clocks_and_plls,
    gpio::{self},
    pac::{CorePeripherals, Peripherals},
    Clock, Sio, Watchdog,
};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

static WASM: &[u8] =
    include_bytes!("../../../projects/blink/target/wasm32-unknown-unknown/release/blink.wasm")
        .as_slice();

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 10 * 1024; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

    let core = CorePeripherals::take().unwrap();
    let mut p = Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(p.WATCHDOG);
    let sio = Sio::new(p.SIO);

    // External high-speed crystal is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        p.XOSC,
        p.CLOCKS,
        p.PLL_SYS,
        p.PLL_USB,
        &mut p.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = gpio::Pins::new(p.IO_BANK0, p.PADS_BANK0, sio.gpio_bank0, &mut p.RESETS);

    let wasm = Wasm::parse(WASM).unwrap();
    let mut process = wasm.spawn("start").unwrap();

    let mut state = State {
        leds: &mut [
            &mut pins.gpio1.into_push_pull_output(),
            &mut pins.gpio2.into_push_pull_output(),
            &mut pins.gpio3.into_push_pull_output(),
            &mut pins.gpio4.into_push_pull_output(),
            &mut pins.gpio5.into_push_pull_output(),
        ],
        delay: Delay::new(core.SYST, clocks.system_clock.freq().integer()),
    };
    loop {
        match process.step().unwrap() {
            ProcessAction::None => {}
            ProcessAction::Finished(_) => todo!(),
            ProcessAction::CallExtern { function, args } => {
                embedded_wasm::FfiHandler::handle(&mut state, &mut process, function, args);
            }
        }
    }
}

pub struct State<'a> {
    pub leds: &'a mut [&'a mut dyn OutputPin<Error = Infallible>],
    pub delay: Delay,
}

embedded_wasm::derive_ffi_handler! {
    impl<'a> State<'a> {
        fn get_led_handle(&self, led: i32) -> i32 {
            let idx = shared::LedIndex::from_i32(led).unwrap_or(shared::LedIndex::Unknown);
            idx as i32
        }

        fn led_on(&mut self, idx: i32) {
            if let Some(led) = self.leds.get_mut(idx as usize) {
                led.set_high().unwrap();
            }
        }

        fn led_off(&mut self, idx: i32) {
            if let Some(led) = self.leds.get_mut(idx as usize) {
                led.set_low().unwrap();
            }
        }

        fn delay(&mut self, sleep_ms: i32) {
            self.delay.delay_ms(sleep_ms as u32);
        }

        #[unhandled]
        fn unhandled(&mut self, _fn_name: &str, _args: Vec<embedded_wasm::Dynamic>) {
            // Do nothing
        }
    }
}

#[cfg(not(test))]
use core::alloc::Layout;
#[cfg(not(test))]
#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // let message = alloc::string::ToString::to_string(&info);
    cortex_m::asm::bkpt();
    loop {
        cortex_m::asm::bkpt();
    }
}
