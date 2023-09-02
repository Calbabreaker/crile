use std::time::{Duration, Instant};

pub struct Time {
    last_frame: Instant,
    start: Instant,
    delta: Duration,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            start: Instant::now(),
            delta: Duration::ZERO,
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        self.delta = self.last_frame.elapsed();
        self.last_frame = Instant::now();
    }

    pub fn since_start(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    pub fn delta(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    pub fn framerate(&self) -> f32 {
        1. / self.delta()
    }
}
