use crate::block::Block;
use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::model::ModelVertex;
use wgpu::util::DeviceExt;
use crate::chunk::ChunkPos;
use crate::world;

pub struct ChunkMesh {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

pub struct GpuChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub position: ChunkPos,

    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl GpuChunkMesh {
    pub fn new(
        device: &wgpu::Device,
        mesh: &ChunkMesh,
        position: ChunkPos,
        chunk_bind_group_layout: &wgpu::BindGroupLayout,
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

        let world_position = position.world_origin();

        let uniform = crate::ChunkUniform {
            position: world_position,
            _padding: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Uniform Buffer"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: chunk_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
                label: Some("chunk_bind_group"),
            },
        );

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
            position,
            uniform_buffer,
            bind_group,
        }
    }
}

impl GpuChunkMesh {
    pub fn rebuild(
        &mut self,
        device: &wgpu::Device,
        mesh: &ChunkMesh,
    ) {
        self.vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        self.index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        self.num_indices = mesh.indices.len() as u32;
    }
}

pub fn build_chunk_mesh(
    chunk: &Chunk,
    world: &world::World,
) -> ChunkMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);

                if block == Block::Air {
                    continue;
                }

                if get_block(chunk, world, x as i32 + 1, y as i32, z as i32) == Block::Air {
    add_face(&mut vertices, &mut indices, x as f32, y as f32, z as f32, Face::Right);
}

if get_block(chunk, world, x as i32 - 1, y as i32, z as i32) == Block::Air {
    add_face(&mut vertices, &mut indices, x as f32, y as f32, z as f32, Face::Left);
}

if get_block(chunk, world, x as i32, y as i32 + 1, z as i32) == Block::Air {
    add_face(&mut vertices, &mut indices, x as f32, y as f32, z as f32, Face::Top);
}

if get_block(chunk, world, x as i32, y as i32 - 1, z as i32) == Block::Air {
    add_face(&mut vertices, &mut indices, x as f32, y as f32, z as f32, Face::Bottom);
}

if get_block(chunk, world, x as i32, y as i32, z as i32 + 1) == Block::Air {
    add_face(&mut vertices, &mut indices, x as f32, y as f32, z as f32, Face::Front);
}

if get_block(chunk, world, x as i32, y as i32, z as i32 - 1) == Block::Air {
    add_face(&mut vertices, &mut indices, x as f32, y as f32, z as f32, Face::Back);
}
            }
        }
    }
    

    ChunkMesh { vertices, indices }
}

fn get_block(
    chunk: &Chunk,
    world: &world::World,
    x: i32,
    y: i32,
    z: i32,
) -> Block {
    // Inside this chunk
    if x >= 0
        && x < CHUNK_SIZE as i32
        && y >= 0
        && y < CHUNK_SIZE as i32
        && z >= 0
        && z < CHUNK_SIZE as i32
    {
        return chunk.get(
            x as usize,
            y as usize,
            z as usize,
        );
    }

    // Outside this chunk: determine which neighboring chunk
    // contains the requested block.
    let mut chunk_pos = chunk.position;

    let mut local_x = x;
    let mut local_y = y;
    let mut local_z = z;

    if local_x < 0 {
        chunk_pos.x -= 1;
        local_x += CHUNK_SIZE as i32;
    } else if local_x >= CHUNK_SIZE as i32 {
        chunk_pos.x += 1;
        local_x -= CHUNK_SIZE as i32;
    }

    if local_y < 0 {
        chunk_pos.y -= 1;
        local_y += CHUNK_SIZE as i32;
    } else if local_y >= CHUNK_SIZE as i32 {
        chunk_pos.y += 1;
        local_y -= CHUNK_SIZE as i32;
    }

    if local_z < 0 {
        chunk_pos.z -= 1;
        local_z += CHUNK_SIZE as i32;
    } else if local_z >= CHUNK_SIZE as i32 {
        chunk_pos.z += 1;
        local_z -= CHUNK_SIZE as i32;
    }

    match world.get_chunk(chunk_pos) {
        Some(neighbor) => neighbor.get(
            local_x as usize,
            local_y as usize,
            local_z as usize,
        ),
        None => Block::Air,
    }
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