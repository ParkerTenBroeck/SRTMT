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
                println!("{} ON A NEW THREAD", i);
                // panic!();
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
