#[no_mangle]
#[naked]
#[link_section = ".text.start"]
extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm! {
            ".set noat",

            // initialize stack and gp and fp
            "la $gp, _gp",
            "la $sp, _sp ",
            "move $fp, $sp",

            //initialize bss
            "la $a0, _bss_start",
            "li $a1, 0",
            "la $a2, _bss_size",
            "jal memset",

            "jal main",
            "1:",
            "syscall 0",
            "b 1b",
            options(noreturn),
        }
    }
}

extern "C" {
    pub fn main();
}

#[panic_handler]
#[cfg(feature = "provide_panic_handler")]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::println!("PANIC AT THE DISCO: {:#?}", info);
    loop {
        crate::process::exit(-1);
    }
}
