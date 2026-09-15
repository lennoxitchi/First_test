use crate::block::Block;
use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::model::ModelVertex;
use wgpu::util::DeviceExt;

pub struct ChunkMesh {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

pub struct GpuChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl GpuChunkMesh {
    pub fn new(
        device: &wgpu::Device,
        mesh: &ChunkMesh,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
        }
    }
}

pub fn build_chunk_mesh(chunk: &Chunk) -> ChunkMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);

                if block == Block::Air {
                    continue;
                }

                // Check each of the 6 sides.
                if is_air(chunk, x as i32 + 1, y as i32, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        x as f32,
                        y as f32,
                        z as f32,
                        Face::Right,
                    );
                }

                if is_air(chunk, x as i32 - 1, y as i32, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        x as f32,
                        y as f32,
                        z as f32,
                        Face::Left,
                    );
                }

                if is_air(chunk, x as i32, y as i32 + 1, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        x as f32,
                        y as f32,
                        z as f32,
                        Face::Top,
                    );
                }

                if is_air(chunk, x as i32, y as i32 - 1, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        x as f32,
                        y as f32,
                        z as f32,
                        Face::Bottom,
                    );
                }

                if is_air(chunk, x as i32, y as i32, z as i32 + 1) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        x as f32,
                        y as f32,
                        z as f32,
                        Face::Front,
                    );
                }

                if is_air(chunk, x as i32, y as i32, z as i32 - 1) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        x as f32,
                        y as f32,
                        z as f32,
                        Face::Back,
                    );
                }
            }
        }
    }
    

    ChunkMesh { vertices, indices }
}

fn is_air(chunk: &Chunk, x: i32, y: i32, z: i32) -> bool {
    if x < 0
        || x >= CHUNK_SIZE as i32
        || y < 0
        || y >= CHUNK_SIZE as i32
        || z < 0
        || z >= CHUNK_SIZE as i32
    {
        return true;
    }

    chunk.get(x as usize, y as usize, z as usize) == Block::Air
}

#[derive(Clone, Copy)]
enum Face {
    Right,
    Left,
    Top,
    Bottom,
    Front,
    Back,
}

fn add_face(
    vertices: &mut Vec<ModelVertex>,
    indices: &mut Vec<u32>,
    x: f32,
    y: f32,
    z: f32,
    face: Face,
) {
    let start = vertices.len() as u32;

    let (positions, normal) = match face {
        Face::Right => (
            [
                [x + 1.0, y, z],
                [x + 1.0, y + 1.0, z],
                [x + 1.0, y + 1.0, z + 1.0],
                [x + 1.0, y, z + 1.0],
            ],
            [1.0, 0.0, 0.0],
        ),

        Face::Left => (
            [
                [x, y, z + 1.0],
                [x, y + 1.0, z + 1.0],
                [x, y + 1.0, z],
                [x, y, z],
            ],
            [-1.0, 0.0, 0.0],
        ),

        Face::Top => (
            [
                [x, y + 1.0, z],
                [x, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z],
            ],
            [0.0, 1.0, 0.0],
        ),

        Face::Bottom => (
            [
                [x, y, z + 1.0],
                [x, y, z],
                [x + 1.0, y, z],
                [x + 1.0, y, z + 1.0],
            ],
            [0.0, -1.0, 0.0],
        ),

        Face::Front => (
            [
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y + 1.0, z + 1.0],
                [x, y + 1.0, z + 1.0],
                [x, y, z + 1.0],
            ],
            [0.0, 0.0, 1.0],
        ),

        Face::Back => (
            [
                [x, y, z],
                [x, y + 1.0, z],
                [x + 1.0, y + 1.0, z],
                [x + 1.0, y, z],
            ],
            [0.0, 0.0, -1.0],
        ),
    };

    let tex_coords = [
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
    ];

    for i in 0..4 {
        vertices.push(ModelVertex {
            position: positions[i],
            tex_coords: tex_coords[i],
            normal,
        });
    }

    indices.extend_from_slice(&[
        start,
        start + 1,
        start + 2,
        start,
        start + 2,
        start + 3,
    ]);
}