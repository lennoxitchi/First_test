use crate::{
    block::Block,
    chunk::{Chunk, ChunkPos, CHUNK_SIZE},
};
use std::ops::Mul;

pub fn generate_chunk(position: ChunkPos) -> Chunk {
    let mut chunk = Chunk::new(position);

    let world_x: i64 = chunk.position.x * 16;

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let xx = x as i64;
            let xxx = (xx+world_x);
            let ychange = ((xxx as f64).mul(0.2).abs().sin() * 2.2 + 2.2) as usize;
            for y in 0..ychange {
                chunk.set(x, y, z, Block::Stone);
            }

            for y in ychange..4+ychange {
                chunk.set(x, y, z, Block::Dirt);
            }

            chunk.set(x, 4+ychange, z, Block::Grass);
        }
    }

    chunk
}