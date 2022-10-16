


pub fn exit(code: i32) -> ! {
    loop{
        unsafe{
            crate::arch::syscall_s_v::<0>(code as u32);
        }
    }
}