use crate::block::Block;
use crate::block_model::{BlockModel, CubeModel};
use crate::texture::TextureAtlas;

#[derive(Clone, Copy, Debug)]
pub struct BlockRegistry {
    models: [Option<BlockModel>; 4],
}

impl BlockRegistry {
    pub fn new(atlas: &TextureAtlas) -> anyhow::Result<Self> {
        let stone = atlas.index("stone")?;
        let dirt = atlas.index("dirt")?;
        let grass_side = atlas.index("grass_side")?;
        let grass_top = atlas.index("grass_top")?;

        Ok(Self {
            models: [
                None, // Air
                Some(BlockModel::Cube(CubeModel::all(stone))), // Stone
                Some(BlockModel::Cube(CubeModel::new(
                    grass_side,
                    grass_side,
                    grass_side,
                    grass_side,
                    grass_top,
                    dirt,
                ))), // Grass
                Some(BlockModel::Cube(CubeModel::all(dirt))), // Dirt
            ],
        })
    }

    pub fn model(&self, block: Block) -> Option<BlockModel> {
        self.models[block as usize]
    }
}