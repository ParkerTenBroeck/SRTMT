#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm_experimental_arch)]
#![feature(strict_provenance)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(allow_internal_unstable)]
#![feature(linkage)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(box_syntax)]

// #[cfg(not(nightly))]
// compile_error!("ONLY NIGHTLY SUPPORTED");

// #[cfg(not(target = "mips"))]
// compile_error!("ONLY MIPS ARCHITECTURE SUPPORTED");

// #[cfg(not(target_endian = "little"))]
// compile_error!("NOT LITTLE ENDIAN");

pub mod arch;
pub mod core_rust;
pub mod io;
pub mod process;
pub mod sync;
pub mod thread;
pub mod time;

mod marcos;
pub use marcos::*;



#[no_mangle]
#[naked]
#[link_section = ".text.start"]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm! {
            ".set noat",
            "la $gp, _gp",
            "la $sp, _sp ",
            "move $fp, $sp",
            "jal main",
            "1:",
            "syscall 0",
            "b 1b", options(noreturn),
        }
    }
}

extern "C" {
    pub fn main();
}
