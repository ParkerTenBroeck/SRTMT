#![no_std]
#![no_main]

use mlib::*;

#[no_mangle]
fn main() {
    for _ in 0..10 {
        println!("Hello World!");
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {:#?}", info);
    loop {
        mlib::process::exit(-1)
    }
}
