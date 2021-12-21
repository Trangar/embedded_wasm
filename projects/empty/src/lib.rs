#![no_std]

use sys::{delay, embedded_time::duration::Extensions, Led};

#[no_mangle]
pub extern "C" fn start() {
    let mut led = Led::a0();
    loop {
        led.on();
        delay(1000.milliseconds());
        led.off();
        delay(1000.milliseconds());
    }
}
