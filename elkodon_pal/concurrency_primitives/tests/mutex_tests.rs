use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::{Duration, Instant},
};

use elkodon_bb_testing::assert_that;
use elkodon_pal_concurrency_primitives::mutex::*;

const TIMEOUT: Duration = Duration::from_millis(25);

#[test]
fn mutex_lock_blocks() {
    let sut = Mutex::new();
    let counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        sut.try_lock();

        let t1 = s.spawn(|| {
            sut.lock(|_, _| true);
            counter.fetch_add(1, Ordering::Relaxed);
            sut.unlock(|_| {});
        });

        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        sut.unlock(|_| {});

        assert_that!(t1.join(), is_ok);
        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
    });
}

#[test]
fn mutex_lock_with_timeout_blocks() {
    let sut = Mutex::new();
    let counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        sut.try_lock();

        let t1 = s.spawn(|| {
            let lock_result = sut.lock(|atomic, value| {
                let start = Instant::now();
                while atomic.load(Ordering::Relaxed) == *value {
                    if start.elapsed() > TIMEOUT * 2 {
                        return false;
                    }
                }

                true
            });
            counter.fetch_add(1, Ordering::Relaxed);
            sut.unlock(|_| {});
            assert_that!(lock_result, eq true);
        });

        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        sut.unlock(|_| {});

        assert_that!(t1.join(), is_ok);
        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
    });
}

#[test]
fn mutex_lock_with_timeout_and_fails_after_timeout() {
    let sut = Mutex::new();

    sut.try_lock();

    assert_that!(!sut.lock(|atomic, value| {
        let start = Instant::now();
        while atomic.load(Ordering::Relaxed) == *value {
            if start.elapsed() > TIMEOUT {
                return false;
            }
        }

        true
    }), eq true);
}
