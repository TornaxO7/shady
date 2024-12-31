use std::time::{Duration, Instant};

const DEFAULT_REFRESH_TIME: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
pub struct Timer {
    refresh_time: Duration,
    last_time: Duration,
    curr_time: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            refresh_time: DEFAULT_REFRESH_TIME,
            last_time: Duration::from_secs(0),
            curr_time: Instant::now(),
        }
    }

    pub fn ease_time(&mut self) -> Option<f32> {
        let delta_time = self.curr_time.elapsed().abs_diff(self.last_time);

        if delta_time < self.refresh_time {
            Some((delta_time.as_secs_f32() / self.refresh_time.as_secs_f32()).min(1.))
        } else {
            self.last_time = self.curr_time.elapsed();
            None
        }
    }
}
