#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memset(data: *mut u8, val: u8, size: usize) -> *mut core::ffi::c_void {
    for i in 0..size {
        *data.add(i) = val;
    }
    core::mem::transmute(data)
}

#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *mut u8, size: usize) {
    for i in 0..size {
        *dest.add(i) = *src.add(i);
    }
}

#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memcmp(str1: *mut u8, str2: *mut u8, size: usize) -> core::ffi::c_int {
    for i in 0..size {
        match (*str1.add(i)).cmp(&*str2.add(i)) {
            core::cmp::Ordering::Less => return -1,
            core::cmp::Ordering::Equal => return 1,
            core::cmp::Ordering::Greater => continue,
        }
    }
    0
}

#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memmove(mut dest: *mut u8, mut src: *mut u8, count: usize) {
    if src.addr() < dest.addr() {
        dest = dest.add(count);
        src = src.add(count);
        for _ in 0..count {
            dest = dest.sub(1);
            src = src.sub(1);
            *dest = *src;
        }
    } else {
        for _ in 0..count {
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let mut wrapper = $crate::core_rust::DirectWritter::new();
        let _ = core::fmt::Write::write_fmt(&mut wrapper, core::format_args!($($arg)*));
    };
}
#[macro_export]
#[allow_internal_unstable(format_args_nl)]
macro_rules! println {
    ($($arg:tt)*) => {
        let mut wrapper = $crate::core_rust::DirectWritter::new();
        let _ = core::fmt::Write::write_fmt(&mut wrapper, core::format_args_nl!($($arg)*));
    };
}

#[derive(Default)]
pub struct DirectWritter {}

impl DirectWritter {
    pub fn new() -> Self {
        Self {}
    }
}

impl core::fmt::Write for DirectWritter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::sys::print_str_bytes(s.as_bytes());
        Ok(())
    }
}
