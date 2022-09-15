use std::{time::{SystemTime, Duration}, task, pin::Pin};

use futures::Future;

pub struct IntervalEnsurer {
    end: SystemTime,
    duration: Duration,
}

impl IntervalEnsurer {
    pub const fn new(duration: Duration) -> Self {
        Self { end: SystemTime::UNIX_EPOCH, duration }
    }
}

impl Future for IntervalEnsurer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        cx.waker().clone().wake();
        let now = SystemTime::now();
        if now >= self.end {
            self.get_mut().end = now + self.duration;
            task::Poll::Ready(())
        } else {
            task::Poll::Pending
        }
    }
}
