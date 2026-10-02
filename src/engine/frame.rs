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
    pub fn begin(&mut self, cap: Option<u32>, vsync: bool) -> f32 {
        if self.cap == cap {
            if let Some(rate) = cap {
                // With VSync the buffer swap already waits for the display.
                // Aim slightly early so a cap equal to the refresh rate never
                // drifts past a vertical blank and drops a frame.
                let period = if vsync { 0.98 } else { 1. } / rate.max(1) as f64;
                let deadline = self.last + Duration::from_secs_f64(period);
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
