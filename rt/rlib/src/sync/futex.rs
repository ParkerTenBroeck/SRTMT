use core::sync::atomic::AtomicU32;

use crate::arch::syscall_ss_s;
use crate::arch::FUTEX_WAIT;
use crate::arch::FUTEX_WAKE;

pub struct Futex {
    futex: AtomicU32,
}

impl Futex {
    pub fn wake_one(&self) {
        unsafe {
            syscall_ss_s::<FUTEX_WAKE>(&self.futex as *const AtomicU32 as u32, 1);
        }
    }

    pub fn wake_all(&self) {
        unsafe {
            syscall_ss_s::<FUTEX_WAKE>(&self.futex as *const AtomicU32 as u32, u32::MAX);
        }
    }

    pub fn wait(&self, expected: u32) -> bool {
        unsafe { syscall_ss_s::<FUTEX_WAIT>(&self.futex as *const AtomicU32 as u32, expected) != 0 }
    }
}
