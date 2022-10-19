use std::{fmt::Display, num::NonZeroU32};

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
}

pub type Page = [u8; 0x10000];
pub type PageId = usize;
