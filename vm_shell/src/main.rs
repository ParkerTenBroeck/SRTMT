#![no_std]
#![no_main]

use vm_lib::*;

#[no_mangle]
fn main() {
    loop {
        println!("Shell");
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
