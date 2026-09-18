use std::time::{Duration, Instant};

use crate::{State, chunk};

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

        self.accumulator =
            self.accumulator.min(Duration::from_millis(250));

        while self.accumulator >= TICK_DURATION {
            self.tick(state);
            self.accumulator -= TICK_DURATION;
        }
    }

    fn tick(&mut self, state: &mut State) {
        self.receive_chunks(state);
        self.request_chunks(state);
    }

    // ============================================================
    // RECEIVE WORKER RESULTS
    // ============================================================

    fn receive_chunks(&self, state: &mut State) {
    while let Some(result) = state.chunk_worker.try_receive() {
        let position = result.position;

        let is_new_chunk = result.chunk.is_some();

        if is_new_chunk {
            state.pending_chunks.remove(&position);

            if let Some(chunk) = result.chunk {
                state.world.add_chunk(chunk);
            }
        } else {
            state.pending_remeshes.remove(&position);
        }

        // Upload mesh to GPU
        if let Some(existing) = state
            .chunk_meshes
            .iter_mut()
            .find(|mesh| mesh.position == position)
        {
            existing.rebuild(
                &state.device,
                &result.mesh,
            );
        } else {
            let gpu_mesh =
                crate::chunk_mesh::GpuChunkMesh::new(
                    &state.device,
                    &result.mesh,
                    position,
                    &state.chunk_bind_group_layout,
                );

            state.chunk_meshes.push(gpu_mesh);
        }

        // A newly loaded chunk affects all six neighbors.
        if is_new_chunk {
            for neighbor in Self::neighbor_positions(position) {
                if state.world.chunks.contains_key(&neighbor) {
                    Self::queue_remesh(state, neighbor);
                }
            }
        }

        // Something changed while this remesh was pending.
        if state.remesh_again.remove(&position) {
            Self::queue_remesh(state, position);
        }
    }
}

    // ============================================================
    // REQUEST NEW CHUNKS
    // ============================================================

    fn request_chunks(&self, state: &mut State) {
        let position = state.camera.position;

        let chunk_x =
            (position.x / chunk::CHUNK_SIZE as f32).floor() as i64;

        let chunk_z =
            (position.z / chunk::CHUNK_SIZE as f32).floor() as i64;

        let render_distance: i64 = 10;

        let max_per_tick = 50;

        let mut requested = 0;

        for radius in 0..=render_distance {
            for x in -radius..=radius {
                for z in -radius..=radius {
                    if requested >= max_per_tick {
                        return;
                    }

                    // Only process the outer ring.
                    if x.abs() != radius && z.abs() != radius {
                        continue;
                    }

                    let pos = chunk::ChunkPos {
                        x: chunk_x + x,
                        y: 0,
                        z: chunk_z + z,
                    };

                    // Already loaded.
                    if state.world.chunks.contains_key(&pos) {
                        continue;
                    }

                    // Already being generated.
                    if state.pending_chunks.contains(&pos) {
                        continue;
                    }

                    // ------------------------------------------------
                    // Build neighbor snapshots.
                    // ------------------------------------------------

                    let neighbors =
                        Self::get_neighbor_snapshots(
                            state,
                            pos,
                        );

                    // Mark pending BEFORE sending.
                    state.pending_chunks.insert(pos);

                    state.chunk_worker.request(
                        pos,
                        neighbors,
                    );

                    requested += 1;
                }
            }
        }
    }

    // ============================================================
    // REMESH
    // ============================================================

    fn queue_remesh(
    state: &mut State,
    position: chunk::ChunkPos,
) {
    if state.pending_remeshes.contains(&position) {
        // A remesh is already running/queued.
        // Remember that another one is needed after it finishes.
        state.remesh_again.insert(position);
        return;
    }

    let Some(chunk) = state.world.chunks.get(&position) else {
        return;
    };

    let chunk_snapshot =
        chunk::ChunkSnapshot::from(chunk);

    let neighbors =
        Self::get_neighbor_snapshots(state, position);

    state.pending_remeshes.insert(position);

    state.chunk_worker.remesh(
        position,
        chunk_snapshot,
        neighbors,
    );
}

    // ============================================================
    // NEIGHBOR SNAPSHOTS
    // ============================================================

    fn get_neighbor_snapshots(
        state: &State,
        position: chunk::ChunkPos,
    ) -> [Option<chunk::ChunkSnapshot>; 6] {
        [
            // -X
            state.world.chunks.get(
                &chunk::ChunkPos {
                    x: position.x - 1,
                    y: position.y,
                    z: position.z,
                },
            ).map(chunk::ChunkSnapshot::from),

            // +X
            state.world.chunks.get(
                &chunk::ChunkPos {
                    x: position.x + 1,
                    y: position.y,
                    z: position.z,
                },
            ).map(chunk::ChunkSnapshot::from),

            // -Y
            state.world.chunks.get(
                &chunk::ChunkPos {
                    x: position.x,
                    y: position.y - 1,
                    z: position.z,
                },
            ).map(chunk::ChunkSnapshot::from),

            // +Y
            state.world.chunks.get(
                &chunk::ChunkPos {
                    x: position.x,
                    y: position.y + 1,
                    z: position.z,
                },
            ).map(chunk::ChunkSnapshot::from),

            // -Z
            state.world.chunks.get(
                &chunk::ChunkPos {
                    x: position.x,
                    y: position.y,
                    z: position.z - 1,
                },
            ).map(chunk::ChunkSnapshot::from),

            // +Z
            state.world.chunks.get(
                &chunk::ChunkPos {
                    x: position.x,
                    y: position.y,
                    z: position.z + 1,
                },
            ).map(chunk::ChunkSnapshot::from),
        ]
    }

    // ============================================================
    // NEIGHBOR POSITIONS
    // ============================================================

    fn neighbor_positions(
        position: chunk::ChunkPos,
    ) -> [chunk::ChunkPos; 6] {
        [
            chunk::ChunkPos {
                x: position.x - 1,
                y: position.y,
                z: position.z,
            },

            chunk::ChunkPos {
                x: position.x + 1,
                y: position.y,
                z: position.z,
            },

            chunk::ChunkPos {
                x: position.x,
                y: position.y - 1,
                z: position.z,
            },

            chunk::ChunkPos {
                x: position.x,
                y: position.y + 1,
                z: position.z,
            },

            chunk::ChunkPos {
                x: position.x,
                y: position.y,
                z: position.z - 1,
            },

            chunk::ChunkPos {
                x: position.x,
                y: position.y,
                z: position.z + 1,
            },
        ]
    }
}