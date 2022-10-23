pub struct StdOut {}

impl StdOut {
    pub fn new() -> Self {
        Self {}
    }
}

impl core::fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::arch::print_str_bytes(s.as_bytes());
        Ok(())
    }
}
