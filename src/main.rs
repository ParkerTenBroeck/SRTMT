use std::{io::Read, time::Instant};

use core::system::System;

fn main() {
    tracing_subscriber::fmt::init();
    let mut args = std::env::args().skip(1).peekable(); // skip executable name

    let mut system = System::default();
    println!("Loading File");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-B" => {
                if let Some(next) = args.peek() {
                    if next.starts_with('-') {
                        panic!("Expected list of packages to build not: {}", next);
                    }
                    for arg in next.split(',') {
                        let arg = arg.trim();

                        let mut file = std::fs::File::open(arg).unwrap();
                        let mut file_data = Vec::new();
                        let ammount = file.read_to_end(&mut file_data).unwrap();
                        if ammount > 0x10000 {
                            panic!();
                        }

                        system.add_task_with_pages(&[0x0, 0x7FFF], |pages| {
                            let page = &*pages.iter().find(|p| p.1 == 0).unwrap().0;
                            for (index, val) in file_data.iter().enumerate() {
                                page.set_u8((index % 0x10000) as u16, *val);
                            }
                        });
                    }
                }
                args.next();
            }
            _ => {
                panic!("Invalid arguments given: {}", arg);
            }
        }
    }

    println!("Starting");

    let mut system = Box::pin(system);
    // let raw = system.deref_mut() as *mut System;
    // let _ = std::thread::spawn(move || {
    let start = Instant::now();
    let iters = system.run_blocking();
    let dur = start.elapsed();
    println!(
        "All tasks terminated, ran vm for {} iterations in {:?}\nips: {}\nshutting down",
        iters,
        dur,
        iters as f64 / dur.as_secs_f64()
    );
    std::process::exit(0);
    // });

    // let options = eframe::NativeOptions::default();
    // eframe::run_native(
    //     "My egui App",
    //     options,
    //     Box::new(move |_cc| Box::new(MyApp::new(raw))),
    // );
}

// use eframe::egui;

// struct MyApp {
//     raw: *mut System,
// }

// impl MyApp {
//     pub fn new(raw: *mut System) -> Self {
//         Self { raw }
//     }
// }

// impl eframe::App for MyApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("My egui Application");
//             let hell = unsafe { self.raw.as_mut().unwrap() };

//             // ui.horizontal(|ui| {
//             //     ui.label("Your name: ");
//             //     ui.text_edit_singleline(&mut self.name);
//             // });
//             // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
//             // if ui.button("Click each year").clicked() {
//             //     self.age += 1;
//             // }
//             // ui.label(format!("Hello '{}', age {}", self.name, self.age));
//         });
//     }
// }
