use std::time::Duration;

pub struct Fixed {
    duration: Duration,
}

impl Fixed {
    pub fn from_millis(millis: u64) -> Self {
        Fixed {
            duration: Duration::from_millis(millis),
        }
    }
}

impl Iterator for Fixed {
    type Item = Duration;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.duration)
    }
}
