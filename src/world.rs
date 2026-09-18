use std::collections::HashMap;
use std::collections::HashSet;
use crate::{
    block::Block,
    chunk::{Chunk, ChunkPos, CHUNK_SIZE},
    world_generator,
};

pub struct World {
    pub chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
pub fn new() -> Self {
    Self {
        chunks: HashMap::new(),
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
    }
}
}