#![no_std]
#![no_main]

use core::time::Duration;

use mlib::*;

#[no_mangle]
fn main() {
    for i in 0..30 {
        let start = mlib::time::system_time_nanos();
        mlib::thread::sleep(Duration::from_millis(100));
        let end = mlib::time::system_time_nanos();
        println!("{i}: {:?}", Duration::from_nanos(end - start));
    }
    //mlib::process::exit(0);

    for i in 0..50u32 {
        if is_prime(i) {
            println!("{i} is prime");
        }
    }
}

fn is_prime(n: u32) -> bool {
    if n <= 1 {
        return false;
    }
    for a in 2..n {
        if n % a == 0 {
            return false; // if it is not the last statement you need to use `return`
        }
    }
    true // last value to return
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {:#?}", info);
    loop {
        mlib::process::exit(-1)
    }
}
