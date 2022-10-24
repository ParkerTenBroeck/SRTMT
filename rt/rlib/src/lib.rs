#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm_experimental_arch)]
#![feature(strict_provenance)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(allow_internal_unstable)]
#![feature(linkage)]
#![feature(box_syntax)]

#[cfg(not(target_arch = "mips"))]
compile_error!("ONLY MIPS ARCHITECTURE SUPPORTED");
#[cfg(not(target_endian = "little"))]
compile_error!("NOT LITTLE ENDIAN");

pub mod arch;
pub mod core_rust;
pub mod io;
pub mod process;
pub mod sync;
pub mod thread;
pub mod time;

mod marcos;
pub use marcos::*;

pub use core::*;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub use alloc::*; 

pub mod rt;