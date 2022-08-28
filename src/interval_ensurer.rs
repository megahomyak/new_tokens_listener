/// Some magic here. Briefly, it adds a method to the iterators that converts the iterator of `T`
/// into an iterator of `Future<Output = T>`, introducing a delay before you can get the `T`.
use std::{
    marker::PhantomData,
    time::{Duration, SystemTime}, task, pin::Pin,
};

use futures::Future;

pub struct Iterator<'iterator, InnerIter, > {
    inner: InnerIter,
    ensurer: &'iterator mut IntervalEnsurer,
}

pub struct IntervalEnsurerIteratorFuture<'future, T> {
    ensurer: &'future mut IntervalEnsurer,
    return_value: T,
}

impl<'iterator, T, Inner: std::iter::Iterator<Item = T>> std::iter::Iterator
    for Iterator<'iterator, Inner>
{
    type Item = IntervalEnsurerIteratorFuture<'iterator, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|value| IntervalEnsurerIteratorFuture {
                ensurer: self.ensurer,
                return_value: value,
            })
    }
}

impl<'future, T> Future for IntervalEnsurerIteratorFuture<'future, T> {
    type Output = T;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Self::Output> {
        cx.waker().clone().wake();
        if self.ensurer.is_expired() {
            task::Poll::Ready(self.return_value)
        } else {

        task::Poll::Pending
        }
    }
}

pub struct IntervalEnsurer {
    interval_duration: Duration,
    expires_at: SystemTime,
}

impl IntervalEnsurer {
    /// Updates the expiration date if expired.
    pub fn is_expired(&mut self) -> bool {
        let now = SystemTime::now();
        if self.expires_at <= now {
            self.expires_at = now + self.interval_duration;
            true
        } else {
            false
        }
    }
}

trait EnsureInterval<'iterator, InnerIter>: std::iter::Iterator {
    fn ensure_interval(self, ensurer: &'iterator IntervalEnsurer) -> Iterator<InnerIter>;
}

impl<'iterator, T, Iter: std::iter::Iterator<Item = T>> EnsureInterval<'iterator, Iter> for Iter {
    fn ensure_interval(self, ensurer: &'iterator IntervalEnsurer) -> Iterator<Iter> {
        Iterator {
            inner: self,
            ensurer,
        }
    }
}
