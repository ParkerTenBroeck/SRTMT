use std::sync::atomic::{AtomicI16, AtomicI32, AtomicI8, AtomicU16, AtomicU32};
use std::{fmt::Display, num::NonZeroU32, sync::atomic::AtomicU8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(NonZeroU32);

impl Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl ProcessId {
    pub fn new(id: u32) -> Option<Self> {
        Some(Self(NonZeroU32::new(id)?))
    }

    pub fn from_raw(id: u32) -> Self {
        Self(NonZeroU32::new(id).unwrap())
    }

    pub fn into_raw(&self) -> u32 {
        self.0.get()
    }

    pub fn to_tid(&self) -> TaskId {
        TaskId(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(NonZeroU32);

impl PartialEq<ThreadId> for TaskId{
    fn eq(&self, other: &ThreadId) -> bool {
        *self == other.0
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl TaskId {
    pub fn new(id: u32) -> Option<Self> {
        Some(Self(NonZeroU32::new(id)?))
    }

    pub fn from_raw(id: u32) -> Self {
        Self(NonZeroU32::new(id).unwrap())
    }

    pub fn into_raw(&self) -> u32 {
        self.0.get()
    }

    pub fn to_pid(&self) -> ProcessId {
        ProcessId(self.0)
    }
}

impl PartialEq<ProcessId> for TaskId {
    fn eq(&self, other: &ProcessId) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<TaskId> for ProcessId {
    fn eq(&self, other: &TaskId) -> bool {
        self.0 == other.0
    }
}

pub type ThreadId = (TaskId, ProcessId);

#[derive(Debug)]
#[repr(align(0x10000))]
pub struct Page([AtomicU32; 0x10000 >> 2]);

use std::sync::atomic::Ordering::Relaxed;

pub trait CoreAtomic {
    type Atomic;
    type Regular;

    fn store_atomic(val: Self::Regular, atom: &Self::Atomic);
    fn load_atomic(atom: &Self::Atomic) -> Self::Regular;
}

macro_rules! core_atomic {
    ($reg:ty, $atom:ty, unsafe, $get_name:ident, $set_name:ident) => {
        impl CoreAtomic for $reg {
            type Atomic = $atom;
            type Regular = $reg;

            #[inline(always)]
            fn store_atomic(val: Self::Regular, atom: &Self::Atomic) {
                atom.store(val, Relaxed)
            }

            #[inline(always)]
            fn load_atomic(atom: &Self::Atomic) -> Self::Regular {
                atom.load(Relaxed)
            }
        }

        impl Page {
            #[inline(always)]
            /// # Safety `index` must always be properly aligned
            pub unsafe fn $get_name(&self, index: u16) -> $reg {
                self.load_from_core_unchecked::<$reg>(index)
            }

            #[inline(always)]
            /// # Safety `index` must always be properly aligned
            pub unsafe fn $set_name(&self, index: u16, val: $reg) {
                self.set_from_core_unchecked::<$reg>(index, val)
            }
        }
    };

    ($reg:ty, $atom:ty, $get_name:ident, $set_name:ident) => {
        impl CoreAtomic for $reg {
            type Atomic = $atom;
            type Regular = $reg;

            #[inline(always)]
            fn store_atomic(val: Self::Regular, atom: &Self::Atomic) {
                atom.store(val, Relaxed)
            }

            #[inline(always)]
            fn load_atomic(atom: &Self::Atomic) -> Self::Regular {
                atom.load(Relaxed)
            }
        }

        impl Page {
            #[inline(always)]
            pub fn $get_name(&self, index: u16) -> $reg {
                unsafe { self.load_from_core_unchecked::<$reg>(index) }
            }

            #[inline(always)]
            pub fn $set_name(&self, index: u16, val: $reg) {
                unsafe { self.set_from_core_unchecked::<$reg>(index, val) }
            }
        }
    };
}

core_atomic!(u8, AtomicU8, get_u8, set_u8);
core_atomic!(i8, AtomicI8, get_i8, set_i8);
core_atomic!(u16, AtomicU16, unsafe, get_u16_unchecked, set_u16_unchecked);
core_atomic!(i16, AtomicI16, unsafe, get_i16_unchecked, set_i16_unchecked);
core_atomic!(u32, AtomicU32, unsafe, get_u32_unchecked, set_u32_unchecked);
core_atomic!(i32, AtomicI32, unsafe, get_i32_unchecked, set_i32_unchecked);

impl Default for Page {
    fn default() -> Self {
        Page::new()
    }
}

impl Page {
    pub fn new() -> Page {
        Page(unsafe { std::mem::transmute([0xdbdbdbdbu32; 0x10000 >> 2]) })
    }

    #[inline(always)]
    pub unsafe fn set_from_core_unchecked<T: CoreAtomic>(&self, index: u16, val: T::Regular) {
        T::store_atomic(
            val,
            self.0
                .as_ptr()
                .cast::<T::Atomic>()
                .byte_offset(index as isize)
                .as_ref()
                .unwrap_unchecked(),
        )
    }

    #[inline(always)]
    pub unsafe fn load_from_core_unchecked<T: CoreAtomic>(&self, index: u16) -> T::Regular {
        T::load_atomic(
            self.0
                .as_ptr()
                .cast::<T::Atomic>()
                .byte_offset(index as isize)
                .as_ref()
                .unwrap_unchecked(),
        )
    }

    #[inline(always)]
    pub unsafe fn get_from_core_unchecked<T: CoreAtomic>(&self, index: u16) -> &T::Atomic {
        self.0
            .as_ptr()
            .cast::<T::Atomic>()
            .byte_offset(index as isize)
            .as_ref()
            .unwrap_unchecked()
    }
}
