use std::time::{Duration, Instant};

pub struct Time {
    last_frame_time: Instant,
    elapsed: Duration,
    delta: Duration,
    frame_count: u32,
    /// Target delta for normal update and rendering frame rate
    /// Set to None to use unlimited
    target_delta: Option<Duration>,
    /// Target delta for fixed update
    target_fixed_delta: Duration,
    fixed_update_accumulator: Duration,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            last_frame_time: Instant::now(),
            elapsed: Duration::ZERO,
            delta: Duration::ZERO,
            frame_count: 0,
            target_delta: None,
            // By default, 50 times per second
            target_fixed_delta: Duration::from_secs_f32(0.02),
            fixed_update_accumulator: Duration::ZERO,
        }
    }
}

impl Time {
    pub(crate) fn update(&mut self) {
        self.delta = self.last_frame_time.elapsed();
        self.last_frame_time = Instant::now();
        self.frame_count += 1;
        self.fixed_update_accumulator += self.delta;
        self.elapsed += self.delta;
    }

    pub(crate) fn wait_for_target_frame_rate(&self) {
        if let Some(target_delta) = self.target_delta {
            if let Some(wait_duration) = target_delta.checked_sub(self.last_frame_time.elapsed()) {
                std::thread::sleep(wait_duration);
            }
        }
    }

    pub(crate) fn should_call_fixed_update(&mut self) -> bool {
        if self.fixed_update_accumulator >= self.target_fixed_delta {
            self.fixed_update_accumulator -= self.target_fixed_delta;
            true
        } else {
            false
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn set_target_frame_rate(&mut self, target_frame_rate: Option<f32>) {
        log::trace!(
            "Set target frame rate as {} fps",
            target_frame_rate.unwrap_or(f32::INFINITY)
        );
        self.target_delta = target_frame_rate.map(|rate| Duration::from_secs_f32(1. / rate));
    }

    pub fn set_target_fixed_rate(&mut self, target_fixed_rate: f32) {
        self.target_fixed_delta = Duration::from_secs_f32(1. / target_fixed_rate);
    }

    pub fn frame_rate(&self) -> f32 {
        1. / self.delta().as_secs_f32()
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
}
