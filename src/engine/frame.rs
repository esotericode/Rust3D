use std::time::{Duration, Instant};

pub struct FramePacer {
    last: Instant,
    cap: Option<u32>,
}
impl Default for FramePacer {
    fn default() -> Self {
        Self {
            last: Instant::now(),
            cap: None,
        }
    }
}
impl FramePacer {
    /// Limits rendered frames. Simulation still uses FIXED_DT and catches up
    /// through its accumulator. Driver overrides can impose a lower rate.
    pub fn begin(&mut self, cap: Option<u32>) -> f32 {
        if self.cap == cap {
            if let Some(rate) = cap {
                let deadline = self.last + Duration::from_secs_f64(1. / rate.max(1) as f64);
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining > Duration::from_millis(2) {
                    std::thread::sleep(remaining - Duration::from_millis(1));
                }
                while Instant::now() < deadline {
                    std::thread::yield_now();
                }
            }
        }
        self.cap = cap;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f32();
        self.last = now;
        elapsed
    }
}
