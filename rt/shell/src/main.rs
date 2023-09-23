#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use core::time::Duration;

use rlib::*;

#[no_mangle]
fn main() {
    for _i in 0..500 {
        unsafe {
            _ = rlib::thread::create_thread(start, core::ptr::null_mut());
        }
        extern "C" fn start(_args: *mut core::ffi::c_void) -> ! {
            let tstart = rlib::time::system_time_nanos();
            for b in 0..10 {
                let start = rlib::time::system_time_nanos();
                rlib::thread::sleep(Duration::from_millis(1000));
                let end = rlib::time::system_time_nanos();
                println!(
                    "{b} {:?}, {:?}",
                    Duration::from_nanos(end - tstart),
                    Duration::from_nanos(end - start)
                );
            }
            rlib::process::exit(0);
        }
        // let _ = rlib::thread::start_new_thread(move || {
        //     let tstart = rlib::time::systepm_time_nanos();
        //     for b in 0..3 {
        //         let start = rlib::time::system_time_nanos();
        //         rlib::thread::sleep(Duration::from_millis(1000));
        //         let end = rlib::time::system_time_nanos();
        //         println!(
        //             "{i}:{b} {:?}, {:?}",
        //             Duration::from_nanos(end - tstart),
        //             Duration::from_nanos(end - start)
        //         );
        //     }
        // });
    }
    // if true {
    //     rlib::process::exit(0);
    // }

    let number = 4;
    let handle = rlib::thread::spawn(move || {
        for i in 0..number {
            println!("NEW THREAD: {}", i);
            let _ = rlib::thread::spawn(move || {
                for i in 0..50_000u32 {
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

#[global_allocator]
static ALLOCATOR: rt_alloc::Allocator<4096> = rt_alloc::Allocator::new();
