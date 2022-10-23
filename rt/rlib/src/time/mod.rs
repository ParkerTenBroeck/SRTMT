pub fn system_time_nanos() -> u64 {
    unsafe {
        use crate::arch::CURRENT_TIME_NANOS;
        crate::arch::syscall_v_d::<CURRENT_TIME_NANOS>()
    }
}
