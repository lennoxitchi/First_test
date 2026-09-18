use crate::block::Block;
use crate::block_model::BlockModel;
use crate::block_registry::BlockRegistry;
use crate::chunk::{CHUNK_SIZE, Chunk, ChunkSnapshot};
use crate::model::ModelVertex;
use crate::texture::TextureAtlas;
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct MaskCell {
    block: Block,
    positive: bool,
    texture_index: u32,
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
    chunk: &ChunkSnapshot,
    neighbors: &[Option<ChunkSnapshot>; 6],
    registry: &BlockRegistry,
) -> ChunkMesh {
    build_greedy_mesh(
        |x, y, z| {
            get_block(
                chunk,
                neighbors,
                x,
                y,
                z,
            )
        },
        1.0,
        registry,
    )
}

fn cube_texture(
    block: Block,
    axis: usize,
    positive: bool,
    registry: &BlockRegistry,
) -> u32 {
    let Some(BlockModel::Cube(cube)) = registry.model(block) else {
        return 0;
    };

    match (axis, positive) {
        // X
        (0, true) => cube.east,
        (0, false) => cube.west,

        // Y
        (1, true) => cube.top,
        (1, false) => cube.bottom,

        // Z
        (2, true) => cube.south,
        (2, false) => cube.north,

        _ => unreachable!(),
    }
}

fn build_greedy_mesh<F>(
    mut get_block: F,
    scale: f32,
    registry: &BlockRegistry,
) -> ChunkMesh
where
    F: FnMut(i32, i32, i32) -> Block,
{
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for axis in 0..3 {
        let (u_axis, v_axis) = match axis {
            0 => (1, 2),
            1 => (0, 2),
            2 => (0, 1),
            _ => unreachable!(),
        };

        for slice in 0..=CHUNK_SIZE {
            let mut mask: [Option<MaskCell>; CHUNK_SIZE * CHUNK_SIZE] =
                [None; CHUNK_SIZE * CHUNK_SIZE];

            for v in 0..CHUNK_SIZE {
                for u in 0..CHUNK_SIZE {
                    let mut a = [0i32; 3];
                    let mut b = [0i32; 3];

                    a[axis] = slice as i32 - 1;
                    b[axis] = slice as i32;

                    a[u_axis] = u as i32;
                    a[v_axis] = v as i32;

                    b[u_axis] = u as i32;
                    b[v_axis] = v as i32;

                    let block_a =
                        get_block(a[0], a[1], a[2]);

                    let block_b =
                        get_block(b[0], b[1], b[2]);

                    mask[u + v * CHUNK_SIZE] =
                        if block_a != Block::Air
    && block_b == Block::Air
{
    Some(MaskCell {
        block: block_a,
        positive: true,
        texture_index: cube_texture(
            block_a,
            axis,
            true,
            registry,
        ),
    })
} else if block_b != Block::Air
    && block_a == Block::Air
{
    Some(MaskCell {
        block: block_b,
        positive: false,
        texture_index: cube_texture(
            block_b,
            axis,
            false,
            registry,
        ),
    })
} else {
    None
}
                }
            }

            let mut v = 0;

            while v < CHUNK_SIZE {
                let mut u = 0;

                while u < CHUNK_SIZE {
                    let index = u + v * CHUNK_SIZE;

                    let Some(face) = mask[index] else {
                        u += 1;
                        continue;
                    };

                    let block = face.block;
                    let positive = face.positive;
                    let texture_index = face.texture_index;

                    let mut width = 1;

                    while u + width < CHUNK_SIZE {
                        let next =
                            mask[(u + width) + v * CHUNK_SIZE];

                        if next
                            != Some(MaskCell {
                                block,
                                positive,
                                texture_index,
                            })
                        {
                            break;
                        }

                        width += 1;
                    }

                    let mut height = 1;

                    'height: while v + height < CHUNK_SIZE {
                        for x in 0..width {
                            let next =
                                mask[
                                    (u + x)
                                        + (v + height) * CHUNK_SIZE
                                ];

                            if next
                                != Some(MaskCell {
                                    block,
                                    positive,
                                    texture_index,
                                })
                            {
                                break 'height;
                            }
                        }

                        height += 1;
                    }

                    add_greedy_quad_scaled(
                        &mut vertices,
                        &mut indices,
                        axis,
                        slice,
                        u,
                        v,
                        width,
                        height,
                        positive,
                        texture_index,
                        scale,
                    );

                    for y in 0..height {
                        for x in 0..width {
                            mask[
                                (u + x)
                                    + (v + y) * CHUNK_SIZE
                            ] = None;
                        }
                    }

                    u += width;
                }

                v += 1;
            }
        }
    }

    ChunkMesh {
        vertices,
        indices,
    }
}

fn add_greedy_quad_scaled(
    vertices: &mut Vec<ModelVertex>,
    indices: &mut Vec<u32>,
    axis: usize,
    slice: usize,
    u: usize,
    v: usize,
    width: usize,
    height: usize,
    positive: bool,
    texture_index: u32,
    scale: f32,
) {
    let start = vertices.len() as u32;

    // These are the geometry axes used by the greedy mask.
    //
    // X face: rectangle is Y × Z
    // Y face: rectangle is X × Z
    // Z face: rectangle is X × Y
    let (u_axis, v_axis) = match axis {
        0 => (1, 2), // X: U-geometry = Y, V-geometry = Z
        1 => (0, 2), // Y: U-geometry = X, V-geometry = Z
        2 => (0, 1), // Z: U-geometry = X, V-geometry = Y
        _ => unreachable!(),
    };

    let mut origin = [0.0f32; 3];

    origin[axis] = slice as f32 * scale;
    origin[u_axis] = u as f32 * scale;
    origin[v_axis] = v as f32 * scale;

    let mut du = [0.0f32; 3];
    du[u_axis] = width as f32 * scale;

    let mut dv = [0.0f32; 3];
    dv[v_axis] = height as f32 * scale;

    let p0 = origin;

    let p1 = [
        origin[0] + du[0],
        origin[1] + du[1],
        origin[2] + du[2],
    ];

    let p2 = [
        origin[0] + du[0] + dv[0],
        origin[1] + du[1] + dv[1],
        origin[2] + du[2] + dv[2],
    ];

    let p3 = [
        origin[0] + dv[0],
        origin[1] + dv[1],
        origin[2] + dv[2],
    ];

    let normal = {
        let mut n = [0.0f32; 3];
        n[axis] = if positive { 1.0 } else { -1.0 };
        n
    };

    /*
     * Texture coordinates
     *
     * The important part is that every face gets a proper:
     *
     *     U = horizontal
     *     V = vertical
     *
     * orientation.
     *
     * UVs are in BLOCK units. Therefore a 3x2 greedy quad
     * gets UVs covering 3x2 texture repetitions.
     */

    let (positions, tex_coords) = match (axis, positive) {

        // =========================================================
        // EAST (+X)
        //
        // Geometry dimensions:
        //     width  = Y
        //     height = Z
        //
        // Texture:
        //     horizontal = Z
        //     vertical   = Y
        // =========================================================

        (0, true) => (
            [p0, p1, p2, p3],
            [
                [0.0, width as f32],
                [0.0, 0.0],
                [height as f32, 0.0],
                [height as f32, width as f32],
            ],
        ),

        // =========================================================
        // WEST (-X)
        //
        // Same texture orientation as east, but reverse the
        // geometry winding because the normal points -X.
        // =========================================================

        (0, false) => (
            [p0, p3, p2, p1],
            [
                [0.0, width as f32],
                [height as f32, width as f32],
                [height as f32, 0.0],
                [0.0, 0.0],
            ],
        ),

        // =========================================================
        // TOP (+Y)
        //
        // Geometry:
        //     width  = X
        //     height = Z
        //
        // Texture:
        //     horizontal = X
        //     vertical   = Z
        // =========================================================

        (1, true) => (
            [p0, p3, p2, p1],
            [
                [0.0, 0.0],
                [0.0, height as f32],
                [width as f32, height as f32],
                [width as f32, 0.0],
            ],
        ),

        // =========================================================
        // BOTTOM (-Y)
        // =========================================================

        (1, false) => (
            [p0, p1, p2, p3],
            [
                [0.0, 0.0],
                [width as f32, 0.0],
                [width as f32, height as f32],
                [0.0, height as f32],
            ],
        ),

        // =========================================================
        // SOUTH (+Z)
        //
        // Geometry:
        //     width  = X
        //     height = Y
        //
        // Texture:
        //     horizontal = X
        //     vertical   = Y
        // =========================================================

        (2, true) => (
            [p0, p1, p2, p3],
            [
                [0.0, height as f32],
                [width as f32, height as f32],
                [width as f32, 0.0],
                [0.0, 0.0],
            ],
        ),

        // =========================================================
        // NORTH (-Z)
        // =========================================================

        (2, false) => (
            [p0, p3, p2, p1],
            [
                [0.0, height as f32],
                [width as f32, height as f32],
                [width as f32, 0.0],
                [0.0, 0.0],
            ],
        ),

        _ => unreachable!(),
    };

    for i in 0..4 {
        vertices.push(ModelVertex {
            position: positions[i],
            tex_coords: tex_coords[i],
            normal,
            texture_index,
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

fn get_block(
    chunk: &ChunkSnapshot,
    neighbors: &[Option<ChunkSnapshot>; 6],
    x: i32,
    y: i32,
    z: i32,
) -> Block {
    // Inside current chunk
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

    // Only a single axis should be outside here for the
    // calls made by build_chunk_mesh().

    // -X
    if x < 0 {
        if y < 0 || y >= CHUNK_SIZE as i32
            || z < 0 || z >= CHUNK_SIZE as i32
        {
            return Block::Air;
        }

        return neighbors[0]
            .as_ref()
            .map(|neighbor| {
                neighbor.get(
                    CHUNK_SIZE - 1,
                    y as usize,
                    z as usize,
                )
            })
            .unwrap_or(Block::Air);
    }

    // +X
    if x >= CHUNK_SIZE as i32 {
        if y < 0 || y >= CHUNK_SIZE as i32
            || z < 0 || z >= CHUNK_SIZE as i32
        {
            return Block::Air;
        }

        return neighbors[1]
            .as_ref()
            .map(|neighbor| {
                neighbor.get(
                    0,
                    y as usize,
                    z as usize,
                )
            })
            .unwrap_or(Block::Air);
    }

    // -Y
    if y < 0 {
        if x < 0 || x >= CHUNK_SIZE as i32
            || z < 0 || z >= CHUNK_SIZE as i32
        {
            return Block::Air;
        }

        return neighbors[2]
            .as_ref()
            .map(|neighbor| {
                neighbor.get(
                    x as usize,
                    CHUNK_SIZE - 1,
                    z as usize,
                )
            })
            .unwrap_or(Block::Air);
    }

    // +Y
    if y >= CHUNK_SIZE as i32 {
        if x < 0 || x >= CHUNK_SIZE as i32
            || z < 0 || z >= CHUNK_SIZE as i32
        {
            return Block::Air;
        }

        return neighbors[3]
            .as_ref()
            .map(|neighbor| {
                neighbor.get(
                    x as usize,
                    0,
                    z as usize,
                )
            })
            .unwrap_or(Block::Air);
    }

    // -Z
    if z < 0 {
        if x < 0 || x >= CHUNK_SIZE as i32
            || y < 0 || y >= CHUNK_SIZE as i32
        {
            return Block::Air;
        }

        return neighbors[4]
            .as_ref()
            .map(|neighbor| {
                neighbor.get(
                    x as usize,
                    y as usize,
                    CHUNK_SIZE - 1,
                )
            })
            .unwrap_or(Block::Air);
    }

    // +Z
    if z >= CHUNK_SIZE as i32 {
        if x < 0 || x >= CHUNK_SIZE as i32
            || y < 0 || y >= CHUNK_SIZE as i32
        {
            return Block::Air;
        }

        return neighbors[5]
            .as_ref()
            .map(|neighbor| {
                neighbor.get(
                    x as usize,
                    y as usize,
                    0,
                )
            })
            .unwrap_or(Block::Air);
    }

    Block::Air
}