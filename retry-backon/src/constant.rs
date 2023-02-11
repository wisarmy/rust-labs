use std::time::Duration;

use crate::backoff::BackoffBuilder;
#[derive(Debug, Clone)]
pub struct ConstantBuilder {
    dealy: Duration,
    max_times: Option<usize>,
}

impl Default for ConstantBuilder {
    fn default() -> Self {
        Self {
            dealy: Duration::from_secs(1),
            max_times: Some(3),
        }
    }
}

impl ConstantBuilder {
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.dealy = delay;
        self
    }

    pub fn with_max_times(mut self, max_times: usize) -> Self {
        self.max_times = Some(max_times);
        self
    }
}

impl BackoffBuilder for ConstantBuilder {
    type Backoff = ConstantBackoff;
    fn build(&self) -> Self::Backoff {
        ConstantBackoff {
            dealy: self.dealy,
            max_times: self.max_times,
            attempts: 0,
        }
    }
}

pub struct ConstantBackoff {
    dealy: Duration,
    max_times: Option<usize>,

    attempts: usize,
}

impl Default for ConstantBackoff {
    fn default() -> Self {
        Self {
            dealy: Duration::from_secs(1),
            max_times: Some(3),
            attempts: 0,
        }
    }
}

impl Iterator for ConstantBackoff {
    type Item = Duration;
    fn next(&mut self) -> Option<Self::Item> {
        match self.max_times {
            None => Some(self.dealy),
            Some(max_times) => {
                if self.attempts >= max_times {
                    None
                } else {
                    self.attempts += 1;
                    Some(self.dealy)
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::backoff::BackoffBuilder;
    use crate::constant::ConstantBuilder;

    #[test]
    fn test_constant_default() {
        let mut exp = ConstantBuilder::default().build();

        assert_eq!(Some(Duration::from_secs(1)), exp.next());
        assert_eq!(Some(Duration::from_secs(1)), exp.next());
        assert_eq!(Some(Duration::from_secs(1)), exp.next());
        assert_eq!(None, exp.next());
    }

    #[test]
    fn test_constant_with_delay() {
        let mut exp = ConstantBuilder::default()
            .with_delay(Duration::from_secs(2))
            .build();

        assert_eq!(Some(Duration::from_secs(2)), exp.next());
        assert_eq!(Some(Duration::from_secs(2)), exp.next());
        assert_eq!(Some(Duration::from_secs(2)), exp.next());
        assert_eq!(None, exp.next());
    }

    #[test]
    fn test_constant_with_times() {
        let mut exp = ConstantBuilder::default().with_max_times(1).build();

        assert_eq!(Some(Duration::from_secs(1)), exp.next());
        assert_eq!(None, exp.next());
    }
}
