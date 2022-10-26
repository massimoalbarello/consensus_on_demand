use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::RwLock;
use std::time::SystemTime;

/// Time since UNIX_EPOCH (in nanoseconds). Just like 'std::time::Instant' or
/// 'std::time::SystemTime', [Time] does not implement the [Default] trait.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct Time(u64);

impl Time {
    /// A private function to cast from [Duration] to [Time].
    fn from_duration(t: Duration) -> Self {
        Time(t.as_nanos() as u64)
    }
}
impl std::ops::Add<Duration> for Time {
    type Output = Time;
    fn add(self, dur: Duration) -> Time {
        Time::from_duration(Duration::from_nanos(self.0) + dur)
    }
}

/// The unix epoch.
pub const UNIX_EPOCH: Time = Time(0);

/// A interface that represent the source of time.
pub trait TimeSource: Send + Sync {
    /// Return the releative time since origin. The definition of origin depends
    /// on the actual implementation. For [SysTimeSource] it is the UNIX
    /// epoch.
    fn get_relative_time(&self) -> Time;
}

/// Time source using the system time.
pub struct SysTimeSource {
    current_time: RwLock<Time>,
}

/// Provide real system time as a [TimeSource].
impl SysTimeSource {
    /// Create a new [SysTimeSource].
    pub fn new() -> Self {
        SysTimeSource {
            current_time: RwLock::new(system_time_now()),
        }
    }
}

impl TimeSource for SysTimeSource {
    fn get_relative_time(&self) -> Time {
        *self.current_time.read().unwrap()
    }
}

/// Return the current system time. Note that the value returned is not
/// guaranteed to be monotonic.
fn system_time_now() -> Time {
    UNIX_EPOCH
        + SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime is before UNIX EPOCH!")
}
