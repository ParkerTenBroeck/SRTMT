#[inline(always)]
pub fn spin_loop() {
    unsafe {
        use crate::arch::WAIT_CONTINUE;
        // basically ends the scheduled execution early
        crate::arch::syscall_v_v::<WAIT_CONTINUE>();
    }
}

#[inline(always)]
pub const fn black_box<T>(dummy: T) -> T {
    core::hint::black_box(dummy)
}
