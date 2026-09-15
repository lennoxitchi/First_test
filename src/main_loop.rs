use std::time::{Duration, Instant};

use crate::State;

const TPS: u32 = 20;
const TICK_DURATION: Duration =
    Duration::from_nanos(1_000_000_000 / TPS as u64);

pub struct GameLoop {
    last_time: Instant,
    accumulator: Duration,
}

impl GameLoop {
    pub fn new() -> Self {
        Self {
            last_time: Instant::now(),
            accumulator: Duration::ZERO,
        }
    }

    pub fn update(&mut self, state: &mut State) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_time);
        self.last_time = now;

        self.accumulator += delta;

        // Prevent a huge backlog if the application freezes.
        self.accumulator =
            self.accumulator.min(Duration::from_millis(250));

        while self.accumulator >= TICK_DURATION {
            self.tick(state);
            self.accumulator -= TICK_DURATION;
        }
    }

    fn tick(&mut self, state: &mut State) {
        state.tick(1.0 / TPS as f32);
        // 1 / 20 = 0.05 seconds
    }
}