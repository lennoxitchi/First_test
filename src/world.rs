use std::collections::HashMap;

use crate::{
    chunk::{Chunk, ChunkPos},
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
}