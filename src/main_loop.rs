use std::time::{Duration, Instant};

use crate::{State, chunk, world};

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

    pub fn tick(&mut self, state: &mut State) {
    state.tick(1.0 / TPS as f32);

    let position = state.camera.position;

let chunk_x: i64 =
    (position.x / chunk::CHUNK_SIZE as f32).floor() as i64;

let chunk_z: i64 =
    (position.z / chunk::CHUNK_SIZE as f32).floor() as i64;

let render_distance: i64 = 20;

let mut generated: i32 = 0;
let max_per_tick: i32 = 20;

for radius in 0..=render_distance {
    for x in -radius..=radius {
        for z in -radius..=radius {
            if generated >= max_per_tick {
                break;
            }

            // Only generate the outer ring.
            if x.abs() != radius && z.abs() != radius {
                continue;
            }

            let pos = chunk::ChunkPos {
                x: chunk_x + x,
                y: 0,
                z: chunk_z + z,
            };

            if !state.world.chunks.contains_key(&pos) {
                state.world.generate_chunk(pos);

state.mark_chunk_and_neighbors_dirty(pos);

                generated += 1;
            }
        }

        if generated >= max_per_tick {
            break;
        }
    }

    if generated >= max_per_tick {
        break;
    }
}
}
}