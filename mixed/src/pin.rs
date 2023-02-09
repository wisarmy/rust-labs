use std::{
    future::Future,
    task::Poll,
    time::{Duration, Instant},
};

struct RandAlwaysOneFuture;

impl Future for RandAlwaysOneFuture {
    type Output = u16;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(1)
    }
}
#[pin_project::pin_project]
pub struct TimedWarpper<Fut: Future> {
    start: Option<Instant>,
    #[pin]
    future: Fut,
}

impl<Fut: Future> TimedWarpper<Fut> {
    pub fn new(future: Fut) -> Self {
        Self {
            start: None,
            future,
        }
    }
}

impl<Fut: Future> Future for TimedWarpper<Fut> {
    type Output = (Fut::Output, Duration);
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        let start = this.start.get_or_insert_with(Instant::now);
        let inner_poll = this.future.as_mut().poll(cx);
        let elapsed = start.elapsed();

        match inner_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready((output, elapsed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_struct_future() {
        assert_eq!(RandAlwaysOneFuture.await, 1);
    }
    #[tokio::test]
    async fn test_timed_warpper() {
        let timedwarpper = TimedWarpper::new(RandAlwaysOneFuture);
        assert_eq!(timedwarpper.await.0, 1);
    }
}
