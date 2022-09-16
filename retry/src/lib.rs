use std::{
    error::Error as StdError,
    fmt::{Display, Formatter},
    thread::sleep,
    time::Duration,
};

use opresult::OperationResult;

pub mod delay;
mod opresult;

pub fn retry<I, O, R, E, OR>(iterable: I, mut operation: O) -> Result<R, Error<E>>
where
    I: IntoIterator<Item = Duration>,
    O: FnMut() -> OR,
    OR: Into<OperationResult<R, E>>,
{
    retry_with_index(iterable, |_| operation())
}

pub fn retry_with_index<I, O, R, E, OR>(iterable: I, mut operation: O) -> Result<R, Error<E>>
where
    I: IntoIterator<Item = Duration>,
    O: FnMut(u64) -> OR,
    OR: Into<OperationResult<R, E>>,
{
    let mut iterator = iterable.into_iter();
    let mut current_try = 1;
    let mut total_delay = Duration::default();
    loop {
        match operation(current_try).into() {
            OperationResult::Ok(v) => return Ok(v),
            OperationResult::Retry(error) => {
                if let Some(delay) = iterator.next() {
                    sleep(delay);
                    current_try += 1;
                    total_delay += delay;
                } else {
                    return Err(Error {
                        error,
                        total_delay,
                        tries: current_try,
                    });
                }
            }
            OperationResult::Err(error) => {
                return Err(Error {
                    error,
                    total_delay,
                    tries: current_try,
                });
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Error<E> {
    error: E,
    total_delay: Duration,
    tries: u64,
}

impl<E> Display for Error<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl<E> StdError for Error<E>
where
    E: StdError + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

#[cfg(test)]
mod tests {
    use crate::{delay::Fixed, retry};

    #[test]
    fn succeeds_with_fixed_delay() {
        let mut collecttion = vec![1, 2, 3, 4].into_iter();
        let value = retry(Fixed::from_millis(1000).take(2), || {
            match collecttion.next() {
                Some(n) if n == 2 => Ok(n),
                Some(_) => Err("not 2"),
                None => Err("none"),
            }
        })
        .unwrap();
        assert_eq!(value, 2);
    }
}
