#[derive(Clone, Copy, Debug)]
pub struct CubeModel {
    pub north: u32,
    pub south: u32,
    pub east: u32,
    pub west: u32,
    pub top: u32,
    pub bottom: u32,
}

impl CubeModel {
    pub const fn new(
        north: u32,
        south: u32,
        east: u32,
        west: u32,
        top: u32,
        bottom: u32,
    ) -> Self {
        Self {
            north,
            south,
            east,
            west,
            top,
            bottom,
        }
    }

    pub const fn all(texture_index: u32) -> Self {
        Self {
            north: texture_index,
            south: texture_index,
            east: texture_index,
            west: texture_index,
            top: texture_index,
            bottom: texture_index,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BlockModel {
    Cube(CubeModel),
}