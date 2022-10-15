use core::arch::asm;

//Basic mips stuff

/// Does not take reguments nor return anything
///
/// Halts the VM
pub const HALT: u32 = 0;

/// Print a 2's complement i32 to standard output
///
/// Register 4: i32 value
pub const PRINT_DEC_NUMBER: u32 = 1;

/// Print a C-String ending in a \0 byte.
///
/// Register 4: ptr to begining of string
pub const PRINT_C_STRING: u32 = 4;

/// Print a char to standard output
///
/// Register 4: the char to print
pub const PRINT_CHAR: u32 = 5;

/// Sleep for x ms
///
/// Register 4: the number of ms to sleep for
pub const SLEEP_MS: u32 = 50;

/// Sleep for delta x ms
///
/// Register 4: the number of ms to sleep for munis the time it took since the last call
pub const SLEEP_D_MS: u32 = 51;

/// Current time nanos
///
/// Register 2: upper half of nanos
/// Register 3: lower half of nanos
pub const CURRENT_TIME_NANOS: u32 = 60;

/// Generate a random number between xi32 and yi32
///
/// Register 4: xi32 lower bound
/// Register 4: yi32 upper bound
///
/// Register 2: generated random number
pub const GENERATE_THREAD_RANDOM_NUMBER: u32 = 99;

/// Start a new thread
/// 
/// Register 4: Pointer to thread entry
/// Register 5: Pointer to thread arguments
/// 
/// Register 2: Non zero Id of created thread (if zero an error occured) 
pub const START_NEW_THREAD: u32 = 100;

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

//--------------------------------------------------------------

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_v_v<const CALL_ID: u32>() {
    asm!(
        "syscall {0}",
        const(CALL_ID),
    )
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_s_v<const CALL_ID: u32>(arg1: u32) {
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
    )
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_v_s<const CALL_ID: u32>() -> u32 {
    let ret1;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        out("$2") ret1,
    );
    ret1
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_v_ss<const CALL_ID: u32>() -> (u32, u32) {
    let ret1;
    let ret2;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        out("$2") ret1,
        out("$3") ret2,
    );
    (ret1, ret2)
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_v_d<const CALL_ID: u32>() -> u64 {
    let v0: u32;
    let v1: u32;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        lateout("$2") v0,
        lateout("$3") v1,
    );
    ((v1 as u64) << 32) | (v0 as u64)
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_s_s<const CALL_ID: u32>(arg1: u32) -> u32 {
    let ret1;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        out("$2") ret1,
    );
    ret1
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_s_ss<const CALL_ID: u32>(arg1: u32) -> (u32, u32) {
    let ret1;
    let ret2;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        out("$2") ret1,
        out("$3") ret2,
    );
    (ret1, ret2)
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_s_d<const CALL_ID: u32>(arg1: u32) -> u64 {
    let v0: u32;
    let v1: u32;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        lateout("$2") v0,
        lateout("$3") v1,
    );
    ((v1 as u64) << 32) | (v0 as u64)
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_ss_v<const CALL_ID: u32>(arg1: u32, arg2: u32) {
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        in("$5") arg2,
    );
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_sss_v<const CALL_ID: u32>(arg1: u32, arg2: u32, arg3: u32) {
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        in("$5") arg2,
        in("$6") arg3,
    );
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_sss_s<const CALL_ID: u32>(arg1: u32, arg2: u32, arg3: u32) -> u32 {
    let ret1;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        in("$5") arg2,
        in("$6") arg3,
        out("$2") ret1,
    );
    ret1
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_ds_v<const CALL_ID: u32>(arg1: u64, arg2: u32) {
    let a1 = (arg1 >> 32) as u32;
    let a0 = arg1 as u32;
    let a2 = arg2;
    // let arg_2 = (arg2 >> 32) as u32;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") a0,
        in("$5") a1,
        in("$6") a2,
    );
}

/// # Safety
///
/// If you have to read this then you shouldnt be using this. This is a raw System Call, using it
/// incorrectly can break pretty much anything.
#[inline(always)]
pub unsafe fn syscall_ss_s<const CALL_ID: u32>(arg1: u32, arg2: u32) -> u32 {
    let ret1;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        in("$5") arg2,
        out("$2") ret1,
    );
    ret1
}

