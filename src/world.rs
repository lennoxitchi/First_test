use std::collections::HashMap;
use std::collections::HashSet;
use crate::{
    block::Block,
    chunk::{Chunk, ChunkPos, CHUNK_SIZE},
    world_generator,
};

pub struct World {
    pub chunks: HashMap<ChunkPos, Chunk>,
    pub dirty_chunks: HashSet<ChunkPos>,
}

impl World {
    pub fn new() -> Self {
    Self {
        chunks: HashMap::new(),
        dirty_chunks: HashSet::new(),
    }
}

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn generate_chunk(&mut self, position: ChunkPos) {
        let chunk = world_generator::generate_chunk(position);
        self.add_chunk(chunk);
    }

    pub fn get_block(
        &self,
        world_x: i64,
        world_y: i64,
        world_z: i64,
    ) -> Block {
        let chunk_x = world_x.div_euclid(CHUNK_SIZE as i64);
        let chunk_y = world_y.div_euclid(CHUNK_SIZE as i64);
        let chunk_z = world_z.div_euclid(CHUNK_SIZE as i64);

        let local_x = world_x.rem_euclid(CHUNK_SIZE as i64) as usize;
        let local_y = world_y.rem_euclid(CHUNK_SIZE as i64) as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i64) as usize;

        let chunk_pos = ChunkPos {
            x: chunk_x,
            y: chunk_y,
            z: chunk_z,
        };

        match self.get_chunk(chunk_pos) {
            Some(chunk) => chunk.get(local_x, local_y, local_z),
            None => Block::Air,
        }
    }

    pub fn set_block(
    &mut self,
    world_x: i64,
    world_y: i64,
    world_z: i64,
    block: Block,
) {
    let chunk_x = world_x.div_euclid(CHUNK_SIZE as i64);
    let chunk_y = world_y.div_euclid(CHUNK_SIZE as i64);
    let chunk_z = world_z.div_euclid(CHUNK_SIZE as i64);

    let local_x = world_x.rem_euclid(CHUNK_SIZE as i64) as usize;
    let local_y = world_y.rem_euclid(CHUNK_SIZE as i64) as usize;
    let local_z = world_z.rem_euclid(CHUNK_SIZE as i64) as usize;

    let chunk_pos = ChunkPos {
        x: chunk_x,
        y: chunk_y,
        z: chunk_z,
    };

    if let Some(chunk) = self.get_chunk_mut(chunk_pos) {
        chunk.set(local_x, local_y, local_z, block);

        self.dirty_chunks.insert(chunk_pos);
    }
    self.dirty_chunks.insert(chunk_pos);

if local_x == 0 {
    self.dirty_chunks.insert(ChunkPos {
        x: chunk_pos.x - 1,
        y: chunk_pos.y,
        z: chunk_pos.z,
    });
}

if local_x == CHUNK_SIZE - 1 {
    self.dirty_chunks.insert(ChunkPos {
        x: chunk_pos.x + 1,
        y: chunk_pos.y,
        z: chunk_pos.z,
    });
}

if local_y == 0 {
    self.dirty_chunks.insert(ChunkPos {
        x: chunk_pos.x,
        y: chunk_pos.y - 1,
        z: chunk_pos.z,
    });
}

if local_y == CHUNK_SIZE - 1 {
    self.dirty_chunks.insert(ChunkPos {
        x: chunk_pos.x,
        y: chunk_pos.y + 1,
        z: chunk_pos.z,
    });
}

if local_z == 0 {
    self.dirty_chunks.insert(ChunkPos {
        x: chunk_pos.x,
        y: chunk_pos.y,
        z: chunk_pos.z - 1,
    });
}

if local_z == CHUNK_SIZE - 1 {
    self.dirty_chunks.insert(ChunkPos {
        x: chunk_pos.x,
        y: chunk_pos.y,
        z: chunk_pos.z + 1,
    });
}
}
}