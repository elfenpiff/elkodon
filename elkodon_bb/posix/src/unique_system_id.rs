//! Generates a [`UniqueSystemId`] which is in all processes on the current system. The guarantee
//! is that no other process with the same id exists.
//! But it is possible that a process with a specific id terminates and a new process generates
//! the same id.
//!
//! # Example
//!
//! ```
//! use elkodon_bb_posix::unique_system_id::*;
//!
//! struct MyThing {
//!     unique_system_id: UniqueSystemId,
//! }
//!
//! impl MyThing {
//!     fn new() -> Self {
//!         Self {
//!             unique_system_id: UniqueSystemId::new().expect("Failed to create UniqueSystemId")
//!         }
//!     }
//!
//!     fn id(&self) -> u128 {
//!         self.unique_system_id.value()
//!     }
//! }
//! ```

use std::{
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

use elkodon_bb_elementary::enum_gen;
use elkodon_bb_log::fail;
use elkodon_pal_posix::posix;

use crate::{
    clock::Time,
    process::{Process, ProcessId},
    semaphore::ClockType,
};

enum_gen! { UniqueSystemIdCreationError
  entry:
    FailedToAcquireTime
}

/// Creates a system wide unique id. There does not exist another process which has generated the
/// same id. There will never be another process on the same system with the same id.
/// The [`UniqueSystemId`] is generated by the processes current process id and the current system
/// time using the [`ClockType::Monotonic`].
#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct UniqueSystemId {
    value: u128,
}

impl Display for UniqueSystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl UniqueSystemId {
    /// Creates a new system wide unique id
    pub fn new() -> Result<Self, UniqueSystemIdCreationError> {
        static LAST_NANOSECONDS: AtomicU32 = AtomicU32::new(0);

        let msg = "Failed to create UniqueSystemId";
        let pid = Process::from_self().id().value() as u128;
        let mut now;
        let mut previous_nanoseconds = LAST_NANOSECONDS.load(Ordering::Relaxed);

        // It is possible to create the same UniqueSystemId when in the same process concurrently
        // at the same time a UniqueSystemId is created. To prevent this we reacquire the current
        // time when the nanoseconds refraction is equal and only update LAST_NANOSECONDS
        // when no other updated it in between.
        loop {
            now = fail!(from "UniqueSystemId::new()",
                        when Time::now_with_clock(ClockType::default()),
                        with UniqueSystemIdCreationError::FailedToAcquireTime,
                        "{} since the current time could not be acquired.", msg);

            if now.nanoseconds() != previous_nanoseconds {
                match LAST_NANOSECONDS.compare_exchange(
                    previous_nanoseconds,
                    now.nanoseconds(),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(v) => previous_nanoseconds = v,
                }
            }
        }

        Ok(UniqueSystemId {
            value: (pid << 96) | ((now.seconds() as u128) << 32) | now.nanoseconds() as u128,
        })
    }

    /// Returns the underlying value of the new system wide unique id
    pub fn value(&self) -> u128 {
        self.value
    }

    /// Returns the [`ProcessId`] which was used to create the [`UniqueSystemId`]
    pub fn pid(&self) -> ProcessId {
        ProcessId::new((self.value >> 96) as posix::pid_t)
    }

    /// Returns the [`Time`] when the [`UniqueSystemId`] was created
    pub fn creation_time(&self) -> Time {
        let seconds = ((self.value << 32) >> 64) as u64;
        let nanoseconds = ((self.value << 96) >> 96) as u32;

        Time {
            clock_type: ClockType::default(),
            seconds,
            nanoseconds,
        }
    }
}
