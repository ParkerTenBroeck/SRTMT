pub mod scheduler;
pub mod system;
pub mod task;
pub mod taskpool;
pub mod util;

pub use std::time::SystemTime as SystemTime;
pub use std::time::Duration as Duration;

pub fn systime_now() -> SystemTime{
    std::time::SystemTime::now()
}

pub fn wait_for(dur: Duration) {
    std::thread::sleep(dur);
}