use std::{thread, time::Duration};

use crate::backoff::{Backoff, BackoffBuilder};

pub trait BlockingRetryable<B: BackoffBuilder, T, E, F: FnMut() -> Result<T, E>> {
    fn retry(self, builder: &B) -> BlockingRetry<B::Backoff, T, E, F>;
}

impl<B, T, E, F> BlockingRetryable<B, T, E, F> for F
where
    B: BackoffBuilder,
    F: FnMut() -> Result<T, E>,
{
    fn retry(self, builder: &B) -> BlockingRetry<<B as BackoffBuilder>::Backoff, T, E, F> {
        BlockingRetry::new(self, builder.build())
    }
}

pub struct BlockingRetry<B: Backoff, T, E, F: FnMut() -> Result<T, E>> {
    backoff: B,
    retryable: fn(&E) -> bool,
    notify: fn(&E, Duration),
    f: F,
}

impl<B, T, E, F> BlockingRetry<B, T, E, F>
where
    B: Backoff,
    F: FnMut() -> Result<T, E>,
{
    fn new(f: F, backoff: B) -> Self {
        BlockingRetry {
            backoff,
            retryable: |_: &E| true,
            notify: |_: &E, _: Duration| {},
            f,
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

    pub fn call(mut self) -> Result<T, E> {
        loop {
            let result = (self.f)();

            match result {
                Ok(v) => return Ok(v),
                Err(err) => {
                    if !(self.retryable)(&err) {
                        return Err(err);
                    }

                    match self.backoff.next() {
                        None => return Err(err),
                        Some(dur) => {
                            (self.notify)(&err, dur);
                            thread::sleep(dur);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::exponential::ExponentialBuilder;

    use super::*;
    use anyhow::Result;
    fn always_error() -> Result<()> {
        Err(anyhow::anyhow!("test_query meets error"))
    }

    #[test]
    fn test_retry() -> Result<()> {
        let result = always_error
            .retry(&ExponentialBuilder::default().with_min_delay(Duration::from_millis(1)))
            .call();
        assert!(result.is_err());
        assert_eq!("test_query meets error", result.unwrap_err().to_string());
        Ok(())
    }
    #[test]
    fn test_retry_with_not_retryable_error() -> Result<()> {
        let error_times = Mutex::new(0);
        let f = || {
            let mut x = error_times.lock().unwrap();
            *x += 1;
            Err::<(), anyhow::Error>(anyhow::anyhow!("not retryable"))
        };
        let backoff = ExponentialBuilder::default().with_min_delay(Duration::from_millis(1));
        let result = f
            .retry(&backoff)
            .when(|e| e.to_string() == "retryable")
            .call();
        assert!(result.is_err());
        assert_eq!("not retryable", result.unwrap_err().to_string());
        assert_eq!(*error_times.lock().unwrap(), 1);
        Ok(())
    }
    #[test]
    fn test_retry_with_retryable_error() -> Result<()> {
        let error_times = Mutex::new(0);

        let f = || {
            println!("I have been called!");
            let mut x = error_times.lock().unwrap();
            *x += 1;
            Err::<(), anyhow::Error>(anyhow::anyhow!("retryable"))
        };
        let backoff = ExponentialBuilder::default().with_min_delay(Duration::from_millis(1));
        let result = f
            .retry(&backoff)
            .when(|e| e.to_string() == "retryable")
            .call();
        assert!(result.is_err());
        assert_eq!("retryable", result.unwrap_err().to_string());
        assert_eq!(*error_times.lock().unwrap(), 4);

        Ok(())
    }
}
