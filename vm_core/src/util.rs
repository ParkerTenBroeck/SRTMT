use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(u32);

impl Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl ProcessId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }

    pub fn into_raw(&self) -> u32 {
        self.0
    }
}

pub type Page = [u8; 0x10000];
pub type PageId = usize;
