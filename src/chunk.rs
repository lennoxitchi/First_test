use crate::block::Block;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl ChunkPos {
    pub fn world_origin(&self) -> [f32; 3] {
        [
            self.x as f32 * CHUNK_SIZE as f32,
            self.y as f32 * CHUNK_SIZE as f32,
            self.z as f32 * CHUNK_SIZE as f32,
        ]
    }
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

#[derive(Clone)]
pub struct ChunkSnapshot {
    pub position: ChunkPos,
    pub blocks: Vec<Block>,
}

impl ChunkSnapshot {
    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[Chunk::index(x, y, z)]
    }
}

impl From<&Chunk> for ChunkSnapshot {
    fn from(chunk: &Chunk) -> Self {
        Self {
            position: chunk.position,
            blocks: chunk.blocks.clone(),
        }
    }
}