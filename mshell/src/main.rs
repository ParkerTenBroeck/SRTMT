#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use mlib::*;

#[no_mangle]
fn main() {
    let number = 23;
    let handle = mlib::thread::start_new_thread(move || {
        for i in 0..number {
            println!("NEW THREAD: {}", i);
            let _ = mlib::thread::start_new_thread(move || {
                for i in 0..5_000u32 {
                    if is_prime(i) {
                        println!("{i} is prime");
                    }
                }
            })
            .unwrap();
        }
    })
    .unwrap();
    println!("Thread: {}", handle);

    for i in 0..5 {
        println!("Shell: {}", i);
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
        mlib::process::exit(-1);
    }
}

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<4096> = emballoc::Allocator::new();

extern crate alloc;
