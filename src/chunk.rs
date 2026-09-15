use crate::block::Block;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize =
    CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

pub struct Chunk {
    pub position: ChunkPos,
    pub blocks: Vec<Block>,
}

impl Chunk {
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            blocks: vec![Block::Air; CHUNK_VOLUME],
        }
    }

    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        let index = Self::index(x, y, z);
        self.blocks[index] = block;
    }
}