use std::{io::Read, time::Instant};

use vm_core::system::System;

fn main() {
    tracing_subscriber::fmt::init();

    let mut args = std::env::args().skip(1).peekable(); // skip executable name

    let mut system = System::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-B" => {
                if let Some(next) = args.peek() {
                    if next.starts_with('-') {
                        panic!("Expected list of packages to build not: {}", next);
                    }
                    for arg in next.split(',') {
                        let arg = arg.trim();

                        let pages = system.add_task_with_pages([0x0, 0x7FFF]);
                        let mut file = std::fs::File::open(arg).unwrap();
                        // let mut buf = Vec::new();
                        let ammount = file.read(pages[0]).unwrap();
                        if ammount == 0x10000 {
                            panic!();
                        }
                    }
                }
                args.next();
            }
            _ => {
                panic!("Invalid arguments given: {}", arg);
            }
        }
    }

    let start = Instant::now();
    let iters = system.run_blocking();
    let dur = start.elapsed();
    println!(
        "All tasks terminated, ran vm for {} iterations in {:?}\nshutting down",
        iters, dur
    );
}
