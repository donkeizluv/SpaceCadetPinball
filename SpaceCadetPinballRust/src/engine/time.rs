use std::time::Duration;

#[derive(Debug, Clone)]
pub struct FixedStepper {
    step: Duration,
    accumulator: Duration,
    max_substeps: u32,
}

impl FixedStepper {
    pub fn new(step: Duration) -> Self {
        Self {
            step,
            accumulator: Duration::ZERO,
            max_substeps: 4,
        }
    }

    pub fn with_max_substeps(mut self, max_substeps: u32) -> Self {
        self.max_substeps = max_substeps.max(1);
        self
    }

    pub fn push_frame_time(&mut self, delta: Duration) -> u32 {
        self.accumulator += delta;

        let mut steps = 0;
        while self.accumulator >= self.step && steps < self.max_substeps {
            self.accumulator -= self.step;
            steps += 1;
        }

        if steps == self.max_substeps && self.accumulator >= self.step {
            self.accumulator = Duration::ZERO;
        }

        steps
    }

    pub fn interpolation_alpha(&self) -> f32 {
        if self.step.is_zero() {
            0.0
        } else {
            self.accumulator.as_secs_f32() / self.step.as_secs_f32()
        }
    }
}
