struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;


struct ChunkUniform {
    position: vec3<f32>,
    _padding: f32,
};

@group(2) @binding(0)
var<uniform> chunk: ChunkUniform;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) texture_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texture_index: u32,
};


@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position =
        model.position + chunk.position;

    out.clip_position =
        camera.view_proj *
        vec4<f32>(world_position, 1.0);

    out.tex_coords = model.tex_coords;
    out.normal = model.normal;
    out.texture_index = model.texture_index;

    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;


const ATLAS_COLUMNS: f32 = 3.0;
const ATLAS_ROWS: f32 = 1.0;

fn atlas_uv(
    uv: vec2<f32>,
    texture_index: u32
) -> vec2<f32> {
    let index = f32(texture_index);

    let column = index % ATLAS_COLUMNS;
    let row = floor(index / ATLAS_COLUMNS);

    let tile_size = vec2<f32>(
        1.0 / ATLAS_COLUMNS,
        1.0 / ATLAS_ROWS
    );

    return vec2<f32>(
        column * tile_size.x + uv.x * tile_size.x,
        row * tile_size.y + uv.y * tile_size.y
    );
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = atlas_uv(
        fract(in.tex_coords),
        in.texture_index
    );

    return textureSample(
        t_diffuse,
        s_diffuse,
        uv
    );
}