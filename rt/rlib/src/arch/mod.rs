mod call_ids;
pub use call_ids::*;

mod syscalls;
pub use syscalls::*;

// ---------------------------------------------------------------------------------------

#[inline(always)]
pub fn halt() -> ! {
    unsafe {
        syscall_v_v::<HALT>();
    }

    unsafe {
        core::hint::unreachable_unchecked();
    }
}

#[inline(always)]
pub fn halt_fs() -> ! {
    loop {
        unsafe {
            syscall_v_v::<HALT>();
        }
    }
}

#[inline(always)]
pub fn print_i32(num: i32) {
    unsafe {
        syscall_s_v::<PRINT_DEC_NUMBER>(num as u32);
    }
}

#[inline(always)]
pub fn print_zero_term_str(str: &str) {
    unsafe {
        syscall_s_v::<PRINT_C_STRING>(str.as_ptr().addr() as u32);
    }
}

#[inline(always)]
pub fn print_str(str: &str) {
    for char in str.chars() {
        print_char(char);
    }
}

#[inline(always)]
pub fn print_str_bytes(str: &[u8]) {
    for char in str {
        print_char(*char as char);
    }
}

#[inline(always)]
pub fn print_char(char: char) {
    unsafe {
        syscall_s_v::<PRINT_CHAR>(char as u32);
    }
}

// #[inline(always)]
// pub fn sleep_ms(ms: u32) {
//     unsafe {
//         syscall_1_0::<SLEEP_MS>(ms);
//     }
// }

// #[inline(always)]
// pub fn sleep_d_ms(ms: u32) {
//     unsafe {
//         syscall_1_0::<SLEEP_D_MS>(ms);
//     }
// }

// #[inline(always)]
// pub fn current_time_nanos() -> u64 {
//     unsafe { syscall_0_2_s::<CURRENT_TIME_NANOS>() }
// }

// #[inline(always)]
// pub fn read_i32() -> i32 {
//     unsafe { syscall_0_1::<5>() as i32 }
// }

// pub fn rand_range(min: i32, max: i32) -> i32 {
//     unsafe { syscall_2_1::<GENERATE_THREAD_RANDOM_NUMBER>(min as u32, max as u32) as i32 }
// }

// #[inline(always)]
// pub fn sleep_delta_mills(mills: u32) {
//     unsafe {
//         syscall_1_0::<106>(mills);
//     }
// }

// #[inline(always)]
// pub fn sleep_mills(mills: u32) {
//     unsafe {
//         syscall_1_0::<105>(mills);
//     }
// }

// #[inline(always)]
// pub fn get_micros() -> u64 {
//     unsafe { syscall_0_2_s::<108>() }
// }

// #[inline(always)]
// pub fn get_nanos() -> u64 {
//     unsafe { syscall_0_2_s::<109>() }
// }
