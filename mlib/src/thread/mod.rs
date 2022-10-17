use core::{fmt::Display, num::NonZeroU32, time::Duration};

use crate::arch::START_NEW_THREAD;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
pub fn start_new_thread<F, T>(f: F) -> Result<ThreadJoinHandle, ()>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    use crate::arch::halt_fs;

    let main = move || {
        f();
    };
    let main: Box<dyn FnOnce() + 'static + Send> = box main;
    let p = Box::into_raw(box main);

    let p = p as *mut core::ffi::c_void;
    let res = unsafe { create_thread(run_thread, p) };
    if res.is_err() {
        unsafe {
            //drop if thread isnt created
            let _ = Box::from_raw(p);
        }
    }
    return res;

    extern "C" fn run_thread(main: *mut core::ffi::c_void) {
        unsafe {
            Box::from_raw(main as *mut Box<dyn FnOnce() + Send>)();
        }
        halt_fs();
    }
}

pub unsafe fn create_thread(
    main: extern "C" fn(*mut core::ffi::c_void),
    args: *mut core::ffi::c_void,
) -> Result<ThreadJoinHandle, ()> {
    crate::println!(
        "creator -> main: {:010X}, args: {:010X}",
        main as u32,
        args as u32
    );
    let res = crate::arch::syscall_ss_s::<START_NEW_THREAD>(main as u32, args as u32);
    if let Some(id) = NonZeroU32::new(res) {
        Ok(ThreadJoinHandle { id })
    } else {
        Err(())
    }
}

pub struct ThreadJoinHandle {
    id: NonZeroU32,
}

impl Display for ThreadJoinHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.id)
    }
}


pub fn sleep(dur: Duration) {
    let nanos = dur.as_nanos() as u64;
    unsafe{
        use crate::arch::SLEEP_NANOS;
        crate::arch::syscall_d_v::<SLEEP_NANOS>(nanos);
    }
}