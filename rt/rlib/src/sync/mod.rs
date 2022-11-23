use spin::RelaxStrategy;


pub type Mutex<T> = spin::mutex::Mutex<T, VmRelax>;
pub type RwLock<T> = spin::rwlock::RwLock<T, VmRelax>;
pub type Lazy<T> = spin::Lazy<T, VmRelax>;
pub type Once<T> = spin::Once<T>;
pub type Barrier = spin::barrier::Barrier<VmRelax>;

pub struct VmRelax;

impl RelaxStrategy for VmRelax{
    #[inline(always)]
    fn relax() {
        unsafe{
            
            use crate::arch::WAIT_CONTINUE;
            crate::arch::syscall_v_v::<WAIT_CONTINUE>();
        }
    }
}

