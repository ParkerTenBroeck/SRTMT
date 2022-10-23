pub mod scheduler;
pub mod system;
pub mod task;
pub mod taskpool;
pub mod util;

pub use std::time::Duration;
pub use std::time::SystemTime;

pub fn systime_now() -> SystemTime {
    std::time::SystemTime::now()
}

pub fn wait_for(dur: Duration) {
    //tracing::info!("Sleeping for {:?}", dur);
    std::thread::sleep(dur);
}
