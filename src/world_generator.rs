use crate::{
    block::Block,
    chunk::{Chunk, ChunkPos, CHUNK_SIZE},
};

pub fn generate_chunk(position: ChunkPos) -> Chunk {
    let mut chunk = Chunk::new(position);

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for y in 0..4 {
                chunk.set(x, y, z, Block::Stone);
            }

            for y in 4..7 {
                chunk.set(x, y, z, Block::Dirt);
            }

            chunk.set(x, 7, z, Block::Grass);
        }
    }

    chunk
}