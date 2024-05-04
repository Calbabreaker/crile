use std::time::{Duration, Instant};

pub struct Time {
    last_frame: Instant,
    start: Instant,
    delta: Duration,
    frame_count: u32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            start: Instant::now(),
            delta: Duration::ZERO,
            frame_count: 0,
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        self.delta = self.last_frame.elapsed();
        self.last_frame = Instant::now();
        self.frame_count += 1;
    }

    pub fn since_start(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn frame_rate(&self) -> f32 {
        1. / self.delta().as_secs_f32()
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
}
