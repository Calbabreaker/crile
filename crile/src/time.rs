use std::time::{Duration, Instant};

pub struct Time {
    last_frame: Instant,
    start: Instant,
    delta: Duration,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            start: Instant::now(),
            delta: Duration::ZERO,
        }
    }

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

    pub fn frame_rate(&self) -> f32 {
        1. / self.delta()
    }
}
