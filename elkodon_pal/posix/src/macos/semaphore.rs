#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use elkodon_pal_concurrency_primitives::semaphore::Semaphore;

use crate::posix::pthread::{wait, wake_one};
use crate::posix::Errno;
use crate::posix::*;

pub unsafe fn sem_create(name: *const char, oflag: int, mode: mode_t, value: uint) -> *mut sem_t {
    SEM_FAILED
}

pub unsafe fn sem_post(sem: *mut sem_t) -> int {
    if (*sem).semaphore.value() == u32::MAX {
        Errno::set(Errno::EOVERFLOW);
        return -1;
    }

    (*sem).semaphore.post(wake_one);

    Errno::set(Errno::ESUCCES);
    0
}

pub unsafe fn sem_wait(sem: *mut sem_t) -> int {
    (*sem).semaphore.wait(|atomic, value| -> bool {
        wait(atomic, value);
        true
    });

    Errno::set(Errno::ESUCCES);
    0
}

pub unsafe fn sem_trywait(sem: *mut sem_t) -> int {
    match (*sem).semaphore.try_wait() {
        true => {
            Errno::set(Errno::ESUCCES);
            0
        }
        false => {
            Errno::set(Errno::EAGAIN);
            -1
        }
    }
}

pub unsafe fn sem_timedwait(sem: *mut sem_t, abs_timeout: *const timespec) -> int {
    let mut current_time = timespec::new();
    let mut wait_time = timespec::new();

    loop {
        if sem_trywait(sem) == 0 {
            return 0;
        }

        clock_gettime(CLOCK_REALTIME, &mut current_time);

        if (current_time.tv_sec > (*abs_timeout).tv_sec)
            || (current_time.tv_sec == (*abs_timeout).tv_sec
                && current_time.tv_nsec > (*abs_timeout).tv_nsec)
        {
            Errno::set(Errno::ETIMEDOUT);
            return -1;
        }

        current_time.tv_sec = 0;
        current_time.tv_nsec = 1000000;

        crate::internal::nanosleep(&current_time, &mut wait_time);
    }
}

pub unsafe fn sem_unlink(name: *const char) -> int {
    -1
}

pub unsafe fn sem_open(name: *const char, oflag: int) -> *mut sem_t {
    SEM_FAILED
}

pub unsafe fn sem_close(sem: *mut sem_t) -> int {
    -1
}

pub unsafe fn sem_destroy(sem: *mut sem_t) -> int {
    0
}

pub unsafe fn sem_init(sem: *mut sem_t, pshared: int, value: uint) -> int {
    (*sem).semaphore = Semaphore::new(value as _);
    Errno::set(Errno::ESUCCES);
    0
}
