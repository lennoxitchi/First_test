use crate::block_model::{BlockModel, CubeModel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Stone,
    Grass,
    Dirt,
}

impl Block {
    pub fn model(self) -> Option<BlockModel> {
        match self {
            Block::Air => None,

            Block::Stone => Some(BlockModel::Cube(
                CubeModel::all(2)
            )),

            Block::Dirt => Some(BlockModel::Cube(
                CubeModel::all(1)
            )),

            Block::Grass => Some(BlockModel::Cube(
                CubeModel::new(
                    0, // north
                    0, // south
                    0, // east
                    0, // west
                    0, // top
                    1, // bottom
                )
            )),
        }
    }
}