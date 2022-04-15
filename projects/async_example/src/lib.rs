#![no_std]

use sys::{Led, future, embedded_time::duration::Extensions};

#[no_mangle]
pub extern "C" fn start() {
    future::start_async(async {
        let mut led = Led::d1();
        loop {
            led.on();
            future::sleep(1000.milliseconds()).await;
            led.off();
            future::sleep(1000.milliseconds()).await;
        }
    });
}

