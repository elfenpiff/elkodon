//! Provides an abstraction of [`Process`]es in a POSIX system.
//!
//! # Scheduler & Priorities
//!
//! The priority is independent of the scheduler and 0 is
//! always the lowest and 255 always the highest priority. Internally, the scheduler dependent
//! priority is mapped to the scheduler independent range from 0..255.
//! A disadvantage can arise when the schedulers dependent priority range is either much more
//! fine grained or coarse. But this should be negligible since most scheduler priorities have
//! a range of about 50.
//! The granularity of a [`Scheduler`] can be acquired with [`Scheduler::priority_granularity()`].
//!
//! # Examples
//!
//! ```no_run
//! use elkodon_bb_posix::process::*;
//! use elkodon_bb_posix::scheduler::*;
//!
//! let it_represents_my_process = Process::from_self();
//! let it_represents_my_processes_parent = Process::from_parent();
//! let mut process = Process::from_pid(ProcessId::new(123));
//!
//! process.set_scheduler(Scheduler::Fifo).expect("failed to set scheduler");
//! process.set_priority(100).expect("failed to set priority");
//!
//! println!("pid: {:?}, scheduler: {:?}, prio: {}", process.id(),
//!             process.get_scheduler().expect("failed to get scheduler"),
//!             process.get_priority().expect("failed to get priority"));
//! ```
use std::fmt::Display;

use crate::handle_errno;
use elkodon_bb_elementary::enum_gen;
use elkodon_bb_log::fail;
use elkodon_pal_posix::posix::errno::Errno;
use elkodon_pal_posix::posix::Struct;
use elkodon_pal_posix::*;

use crate::{
    scheduler::{Scheduler, SchedulerConversionError},
    signal::Signal,
};

enum_gen! { ProcessSendSignalError
  entry:
    InsufficientPermissions,
    UnknownProcessId,
    UnknownError(i32)
}

enum_gen! { ProcessGetSchedulerError
  entry:
    InsufficientPermissions,
    UnknownProcessId,
    UnknownError(i32)

  mapping:
    SchedulerConversionError
}

enum_gen! { ProcessSetSchedulerError
  entry:
    InsufficientPermissions,
    UnknownProcessId,
    UnknownError(i32)

  mapping:
    SchedulerConversionError
}

enum_gen! {
    /// The ProcessError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward ProcessError as more generic return value when a method
    /// returns a Process***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    ProcessError
  generalization:
    FailedToSetSchedulerSettings <= ProcessSetSchedulerError,
    FailedToGetSchedulerSettings <= ProcessGetSchedulerError,
    FailedToSendSignal <= ProcessSendSignalError
}

/// Trait to be able to convert integers into processes by interpreting their value as the
/// process id
pub trait ProcessExt {
    fn as_process(&self) -> Process;
}

impl ProcessExt for posix::pid_t {
    fn as_process(&self) -> Process {
        Process::from_pid(ProcessId::new(*self))
    }
}

/// Represents a process id.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProcessId(posix::pid_t);

impl ProcessId {
    /// Creates a new process id.
    pub fn new(value: posix::pid_t) -> Self {
        ProcessId(value)
    }

    /// Returns the underlying integer value of the process id
    pub fn value(&self) -> posix::pid_t {
        self.0
    }
}

impl Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represent a process in a POSIX system.
#[derive(Debug)]
pub struct Process {
    pid: ProcessId,
}

impl Process {
    /// Creates a process from a provided id. The process does not have to exists at the time of
    /// creation. But all other methods may fail when the process does not exist.
    pub fn from_pid(pid: ProcessId) -> Process {
        Process { pid }
    }

    /// Create a process object from own process.
    pub fn from_self() -> Process {
        Process {
            pid: unsafe { ProcessId::new(posix::getpid()) },
        }
    }

    /// Create a process object from the parents process.
    pub fn from_parent() -> Process {
        Process {
            pid: unsafe { ProcessId::new(posix::getppid()) },
        }
    }

    /// Checks if the process is still alive
    pub fn is_alive(&self) -> bool {
        unsafe { posix::kill(self.pid.0, 0_i32) == 0 }
    }

    /// Returns the id of the process.
    pub fn id(&self) -> ProcessId {
        self.pid
    }

    /// Sends a signal to the process.
    pub fn send_signal(&self, signal: Signal) -> Result<(), ProcessSendSignalError> {
        if unsafe { posix::kill(self.pid.0, signal as i32) } == 0 {
            return Ok(());
        }

        let msg = "Unable to send signal to process";
        handle_errno!(ProcessSendSignalError, from self,
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::ESRCH => (UnknownProcessId, "{} since the process does not exist.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
        );
    }

    /// Returns the priority of the process. 0 is the lowest and 255 the highest priority.
    pub fn get_priority(&self) -> Result<u8, ProcessGetSchedulerError> {
        let msg = "Unable to acquire priority of process";
        let scheduler = fail!(from self, when self.get_scheduler(), "{} due to a failure while getting the current scheduler of the process.", msg);

        let mut param = posix::sched_param::new();
        if unsafe { posix::sched_getparam(self.pid.0, &mut param) } == 0 {
            return Ok(scheduler.get_priority_from(&param));
        }

        handle_errno!(ProcessGetSchedulerError, from self,
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::ESRCH => (UnknownProcessId, "{} since the process cannot be found on the system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Set the priority of the process. 0 is the lowest priority and 255 the highest.
    pub fn set_priority(&mut self, priority: u8) -> Result<(), ProcessGetSchedulerError> {
        let msg = "Unable to set process priority";
        let scheduler = fail!(from self, when self.get_scheduler(), "{} due to a failure while acquiring the current process scheduler.", msg);
        let mut param = posix::sched_param::new();
        param.sched_priority = scheduler.policy_specific_priority(priority);

        if unsafe { posix::sched_setparam(self.pid.0, &param) } == 0 {
            return Ok(());
        }

        handle_errno!(ProcessGetSchedulerError, from self,
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::ESRCH => (UnknownProcessId, "{} since the process cannot be found on the system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Returns the currently in use [`Scheduler`] by the process.
    pub fn get_scheduler(&self) -> Result<Scheduler, ProcessGetSchedulerError> {
        let msg = "Unable to acquire scheduler of process";
        let v = unsafe { posix::sched_getscheduler(self.pid.0) };
        if v == 0 {
            return Ok(
                fail!(from self, when Scheduler::from_int(v), "{} since the scheduler seems to be unknown.", msg),
            );
        }

        handle_errno!(ProcessGetSchedulerError, from self,
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::ESRCH => (UnknownProcessId, "{} since the process cannot be found on the system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    /// Sets a new [`Scheduler`] for the process and returns the formerly used [`Scheduler`]
    /// on success.
    pub fn set_scheduler(
        &mut self,
        scheduler: Scheduler,
    ) -> Result<Scheduler, ProcessSetSchedulerError> {
        let msg = "Unable to set scheduler of process";
        let mut param = posix::sched_param::new();
        param.sched_priority = scheduler.policy_specific_priority(0);
        let former_scheduler =
            unsafe { posix::sched_setscheduler(self.pid.0, scheduler as i32, &param) };

        if former_scheduler != -1 {
            return Ok(fail!(from self, when Scheduler::from_int(former_scheduler),
                    "The previous set up scheduler is not supported. New scheduler was successfully set but cannot reverted to previous scheduler."));
        }

        handle_errno!(ProcessSetSchedulerError, from self,
            Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::ESRCH => (UnknownProcessId, "{} since the process cannot be found on the system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
        );
    }
}
