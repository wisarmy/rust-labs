use std::{
    future::Future,
    pin::Pin,
    task::{ready, Poll},
    time::Duration,
};

use pin_project::pin_project;

use crate::backoff::{Backoff, BackoffBuilder};

pub trait Retryable<
    B: BackoffBuilder,
    T,
    E,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
>
{
    fn retry(self, builder: &B) -> Retry<B::Backoff, T, E, Fut, FutureFn>;
}

/// impl Retryable for FutureFn: FnMut()->Future<Output = Result<T, E>>
impl<B, T, E, Fut, FutureFn> Retryable<B, T, E, Fut, FutureFn> for FutureFn
where
    B: BackoffBuilder,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
{
    fn retry(self, builder: &B) -> Retry<B::Backoff, T, E, Fut, FutureFn> {
        Retry::new(self, builder.build())
    }
}
#[pin_project]
pub struct Retry<B: Backoff, T, E, Fut: Future<Output = Result<T, E>>, FutureFn: FnMut() -> Fut> {
    backoff: B,
    retryable: fn(&E) -> bool,
    notify: fn(&E, Duration),
    future_fn: FutureFn,
    #[pin]
    state: State<T, E, Fut>,
}

impl<B, T, E, Fut, FutureFn> Retry<B, T, E, Fut, FutureFn>
where
    B: Backoff,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
{
    fn new(future_fn: FutureFn, backoff: B) -> Self {
        Self {
            backoff,
            retryable: |_: &E| true,
            notify: |_: &E, _: Duration| {},
            future_fn,
            state: State::Idle,
        }
    }
    pub fn when(mut self, retryable: fn(&E) -> bool) -> Self {
        self.retryable = retryable;
        self
    }
    pub fn notify(mut self, notify: fn(&E, Duration)) -> Self {
        self.notify = notify;
        self
    }
}

#[pin_project(project = StateProject)]
enum State<T, E, Fut: Future<Output = Result<T, E>>> {
    Idle,
    Polling(#[pin] Fut),
    Sleeping(#[pin] Pin<Box<tokio::time::Sleep>>),
}

/// impl Future for Retry
impl<B, T, E, Fut, FutureFn> Future for Retry<B, T, E, Fut, FutureFn>
where
    B: Backoff,
    Fut: Future<Output = Result<T, E>>,
    FutureFn: FnMut() -> Fut,
{
    type Output = Result<T, E>;
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        loop {
            let state = this.state.as_mut().project();
            match state {
                StateProject::Idle => {
                    let fut = (this.future_fn)();
                    this.state.set(State::Polling(fut));
                    continue;
                }
                StateProject::Polling(fut) => match ready!(fut.poll(cx)) {
                    Ok(v) => return Poll::Ready(Ok(v)),
                    Err(err) => {
                        if !(this.retryable)(&err) {
                            return Poll::Ready(Err(err));
                        }
                        match this.backoff.next() {
                            None => return Poll::Ready(Err(err)),
                            Some(dur) => {
                                (this.notify)(&err, dur);
                                this.state
                                    .set(State::Sleeping(Box::pin(tokio::time::sleep(dur))));
                                continue;
                            }
                        }
                    }
                },
                StateProject::Sleeping(sl) => {
                    ready!(sl.poll(cx));
                    this.state.set(State::Idle);
                    continue;
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tokio::sync::Mutex;

    use crate::exponential::ExponentialBuilder;

    use super::*;
    async fn always_error() -> Result<()> {
        Err(anyhow::anyhow!("test_query meets error"))
    }
    #[tokio::test]
    async fn test_retry() -> Result<()> {
        let result = always_error
            .retry(&ExponentialBuilder::default().with_min_delay(Duration::from_millis(1)))
            .await;
        assert!(result.is_err());
        assert_eq!("test_query meets error", result.unwrap_err().to_string());

        Ok(())
    }
    #[tokio::test]
    async fn test_retry_with_not_retryable_error() -> Result<()> {
        let error_times = Mutex::new(0);
        let f = || async {
            let mut x = error_times.lock().await;
            *x += 1;
            Err::<(), anyhow::Error>(anyhow::anyhow!("not retryable"))
        };
        let backoff = ExponentialBuilder::default().with_min_delay(Duration::from_millis(1));
        let result = f
            .retry(&backoff)
            .when(|e| e.to_string() == "retryable")
            .await;
        assert!(result.is_err());
        assert_eq!("not retryable", result.unwrap_err().to_string());
        assert_eq!(*error_times.lock().await, 1);

        Ok(())
    }
    #[tokio::test]
    async fn test_retry_with_retryable_error() -> Result<()> {
        let error_times = Mutex::new(0);
        let f = || async {
            let mut x = error_times.lock().await;
            *x += 1;
            Err::<(), anyhow::Error>(anyhow::anyhow!("retryable"))
        };
        let backoff = ExponentialBuilder::default().with_min_delay(Duration::from_millis(1));
        let result = f
            .retry(&backoff)
            .when(|e| e.to_string() == "retryable")
            .await;
        assert!(result.is_err());
        assert_eq!("retryable", result.unwrap_err().to_string());
        assert_eq!(*error_times.lock().await, 4);

        Ok(())
    }
}
