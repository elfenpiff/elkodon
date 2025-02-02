#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;

pub unsafe fn sem_create(name: *const char, oflag: int, mode: mode_t, value: uint) -> *mut sem_t {
    crate::internal::sem_open(name, oflag, mode as core::ffi::c_uint, value)
}

pub unsafe fn sem_post(sem: *mut sem_t) -> int {
    crate::internal::sem_post(sem)
}

pub unsafe fn sem_wait(sem: *mut sem_t) -> int {
    crate::internal::sem_wait(sem)
}

pub unsafe fn sem_trywait(sem: *mut sem_t) -> int {
    crate::internal::sem_trywait(sem)
}

pub unsafe fn sem_timedwait(sem: *mut sem_t, abs_timeout: *const timespec) -> int {
    crate::internal::sem_timedwait(sem, abs_timeout)
}

pub unsafe fn sem_unlink(name: *const char) -> int {
    crate::internal::sem_unlink(name)
}

pub unsafe fn sem_open(name: *const char, oflag: int) -> *mut sem_t {
    crate::internal::sem_open(name, oflag)
}

pub unsafe fn sem_destroy(sem: *mut sem_t) -> int {
    crate::internal::sem_destroy(sem)
}

pub unsafe fn sem_init(sem: *mut sem_t, pshared: int, value: uint) -> int {
    crate::internal::sem_init(sem, pshared, value)
}

pub unsafe fn sem_close(sem: *mut sem_t) -> int {
    crate::internal::sem_close(sem)
}
