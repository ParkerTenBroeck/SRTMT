use core::arch::asm;

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

