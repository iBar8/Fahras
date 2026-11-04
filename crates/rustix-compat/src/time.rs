// SPDX-License-Identifier: MIT

use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub use crate::fs::Timespec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockId {
    Realtime,
    Monotonic,
    Boottime,
    ProcessCPUTime,
    ThreadCPUTime,
    RealtimeCoarse,
    MonotonicCoarse,
}

fn monotonic_origin() -> Instant {
    static ORIGIN: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *ORIGIN.get_or_init(Instant::now)
}

pub fn clock_gettime(id: ClockId) -> Timespec {
    let elapsed = match id {
        ClockId::Realtime | ClockId::RealtimeCoarse => {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()
        }
        _ => monotonic_origin().elapsed(),
    };
    Timespec {
        tv_sec: elapsed.as_secs() as i64,
        tv_nsec: i64::from(elapsed.subsec_nanos()),
    }
}
