#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use core::time::Duration;

use mlib::*;
use spin::RelaxStrategy;

#[no_mangle]
fn main() {

    for i in 0..100 {
        // unsafe {
        //     mlib::thread::create_thread(start, core::ptr::null_mut());
        // }
        // extern "C" fn start(_args: *mut core::ffi::c_void) -> ! {
        //     let tstart = mlib::time::system_time_nanos();
        //     for b in 0..3 {
        //         let start = mlib::time::system_time_nanos();
        //         mlib::thread::sleep(Duration::from_millis(1000));
        //         let end = mlib::time::system_time_nanos();
        //         println!(
        //             "{b} {:?}, {:?}",
        //             Duration::from_nanos(end - tstart),
        //             Duration::from_nanos(end - start)
        //         );
        //     }
        //     mlib::process::exit(0);
        // }
        let _ = mlib::thread::start_new_thread(move || {
            let tstart = mlib::time::system_time_nanos();
            for b in 0..3 {
                let start = mlib::time::system_time_nanos();
                mlib::thread::sleep(Duration::from_millis(1000));
                let end = mlib::time::system_time_nanos();
                println!(
                    "{i}:{b} {:?}, {:?}",
                    Duration::from_nanos(end - tstart),
                    Duration::from_nanos(end - start)
                );
            }
        });
    }
    if true{
        mlib::process::exit(0);
    }

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
