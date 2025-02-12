use std::{
    future::Future,
    sync::Arc,
    task::{Poll, Waker},
    thread,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    rate: u64,
    completed: Arc<Mutex<u64>>,
    reset: Duration,
    remaining: Arc<Mutex<Duration>>,
    now: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(rate: u64, reset: Duration) -> Self {
        Self {
            rate,
            completed: Arc::new(Mutex::new(rate)),
            reset,
            remaining: Arc::new(Mutex::new(reset)),
            now: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn try_wait(&self) -> Result<(), Duration> {
        let mut completed = self.completed.lock();
        if let Some(new_completed) = completed.checked_sub(1) {
            *completed = new_completed;
            return Ok(());
        }

        *completed = self.rate;

        let delta = self.now.lock().elapsed();
        *self.now.lock() = Instant::now();

        let mut remaining = self.remaining.lock();

        match remaining.checked_sub(delta) {
            Some(new_remaining) => {
                *remaining = new_remaining;
                Err(new_remaining)
            }
            _ => {
                *remaining = self.reset;
                Ok(())
            }
        }
    }

    pub async fn wait(&self) {
        RateLimitWait::new(self.clone()).await;
    }
}

struct RateLimitWait {
    waker: Arc<Mutex<Option<Waker>>>,
    limiter: RateLimiter,
}

impl RateLimitWait {
    fn new(limiter: RateLimiter) -> Self {
        let this = Self {
            limiter,
            waker: Arc::new(Mutex::new(None)),
        };

        let waker = this.waker.clone();
        let timeout = this.limiter.reset;

        thread::spawn(move || {
            loop {
                thread::sleep(timeout);
                if let Some(waker) = waker.lock().clone() {
                    waker.wake();
                }
            }
        });

        this
    }
}

impl Future for RateLimitWait {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        *self.waker.lock() = Some(cx.waker().clone());

        match self.limiter.try_wait() {
            Ok(()) => Poll::Ready(()),
            Err(_) => Poll::Pending,
        }
    }
}
