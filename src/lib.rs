mod block;
mod chunk;
mod world;
mod world_generator;
mod main_loop;
mod chunk_mesh;
mod texture;
mod model;
mod resources;
use std::{iter, sync::Arc};
use cgmath::prelude::*;
use texture::*;
use model::*;
use std::ops::Range;
use resources::*;
use std::io::Cursor;
use std::io::BufReader;


use winit::{
    application::ApplicationHandler, event::*, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window,
};

use wgpu::util::DeviceExt;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

const NUM_INSTANCES_PER_ROW: u32 = 1;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 1.0, 0.0, NUM_INSTANCES_PER_ROW as f32 * 1.0);

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {

    let obj_text = load_string(file_name).await?;

    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {

            let mat_text = load_string(&p).await.expect("huh");


            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let obj_materials = obj_materials?;

    let mut materials = Vec::new();

for m in obj_materials {

    let diffuse_texture = if m.diffuse_texture.is_empty() {

        create_white_texture(device, queue)
    } else {
        load_texture(&m.diffuse_texture, device, queue).await?
    };

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &diffuse_texture.view
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(
                    &diffuse_texture.sampler
                ),
            },
        ],
        label: Some("material_bind_group"),
    });

    materials.push(model::Material {
        name: m.name,
        diffuse_texture,
        bind_group,
    });
}
    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    let position = [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ];

                    let tex_coords = if m.mesh.texcoords.is_empty() {
                        [0.0, 0.0]
                    } else {
                        [
                            m.mesh.texcoords[i * 2],
                            1.0 - m.mesh.texcoords[i * 2 + 1],
                        ]
                    };

                    let normal = if m.mesh.normals.is_empty() {
                        [0.0, 1.0, 0.0]
                    } else {
                        [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ]
                    };

                    model::ModelVertex {
                        position,
                        tex_coords,
                        normal,
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{file_name} Vertex Buffer")),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            let index_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{file_name} Index Buffer")),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model {
        meshes,
        materials,
    })
}

// model.rs
pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, material: &'a Material, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );

}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, camera_bind_group: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}




// NEW!
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

// NEW!
impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}


struct CameraController {
    speed: f32,
    sensitivity: f32,

    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,

    fast: bool,

    mouse_dx: f32,
    mouse_dy: f32,
}

impl CameraController {
    fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,

            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,

            fast: false,

            mouse_dx: 0.0,
            mouse_dy: 0.0,
        }
    }

    fn handle_key(&mut self, code: KeyCode, pressed: bool) -> bool {
        match code {
            KeyCode::KeyW => {
                self.forward = pressed;
                true
            }

            KeyCode::KeyS => {
                self.backward = pressed;
                true
            }

            KeyCode::KeyA => {
                self.left = pressed;
                true
            }

            KeyCode::KeyD => {
                self.right = pressed;
                true
            }

            KeyCode::Space => {
                self.up = pressed;
                true
            }

            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.down = pressed;
                true
            }

            KeyCode::ControlLeft | KeyCode::ControlRight => {
                self.fast = pressed;
                true
            }

            _ => false,
        }
    }

    fn handle_mouse(&mut self, dx: f32, dy: f32) {
        self.mouse_dx += dx;
        self.mouse_dy += dy;
    }

    fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        // -------------------------
        // Mouse look
        // -------------------------

        camera.yaw += self.mouse_dx * self.sensitivity;
        camera.pitch -= self.mouse_dy * self.sensitivity;

        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;

        // Prevent flipping upside down
        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;

        camera.pitch = camera.pitch.clamp(
            -max_pitch,
            max_pitch,
        );

        // -------------------------
        // Movement
        // -------------------------

        let speed = if self.fast {
            self.speed * 5.0
        } else {
            self.speed
        };

        let forward = cgmath::Vector3 {
            x: camera.yaw.cos(),
            y: 0.0,
            z: camera.yaw.sin(),
        };

        let right = cgmath::Vector3 {
            x: -forward.z,
            y: 0.0,
            z: forward.x,
        };

        let mut movement = cgmath::Vector3::new(
            0.0,
            0.0,
            0.0,
        );

        if self.forward {
            movement += forward;
        }

        if self.backward {
            movement -= forward;
        }

        if self.right {
            movement += right;
        }

        if self.left {
            movement -= right;
        }

        if self.up {
            movement.y += 1.0;
        }

        if self.down {
            movement.y -= 1.0;
        }

        if !movement.is_zero() {
            movement = movement.normalize();

            camera.position += movement * speed * dt;
        }
    }
}


// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}


struct Camera {
    position: cgmath::Point3<f32>,
    yaw: f32,
    pitch: f32,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let forward = cgmath::Vector3 {
            x: self.yaw.cos() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.sin() * self.pitch.cos(),
        }
        .normalize();

        let right = forward
            .cross(cgmath::Vector3::unit_y())
            .normalize();

        let up = right
            .cross(forward)
            .normalize();

        let view = cgmath::Matrix4::look_to_rh(
            self.position,
            forward,
            up,
        );

        let proj = cgmath::perspective(
            cgmath::Deg(self.fovy),
            self.aspect,
            self.znear,
            self.zfar,
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: Texture,
    obj_model: Model,
    world: world::World,
    chunk_mesh: chunk_mesh::GpuChunkMesh,
}



impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::all(),
    flags: wgpu::InstanceFlags::default(),
    memory_budget_thresholds: Default::default(),
    backend_options: Default::default(),
    display: None,
});

        let surface = instance.create_surface(window.clone()).expect("lib:534 instance.creatsurface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        required_limits: wgpu::Limits::default(),
        memory_hints: Default::default(),
        trace: wgpu::Trace::Off,
    })
    .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");



    let texture_bind_group_layout =
             device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                  entries: &[
                     wgpu::BindGroupLayoutEntry {
                         binding: 0,
                      visibility: wgpu::ShaderStages::FRAGMENT,
                         ty: wgpu::BindingType::Texture {
                             multisampled: false,
                             view_dimension: wgpu::TextureViewDimension::D2,
                             sample_type: wgpu::TextureSampleType::Float { filterable: true },
                         },
                         count: None,
                      },
                    wgpu::BindGroupLayoutEntry {
                          binding: 1,
                          visibility: wgpu::ShaderStages::FRAGMENT,
                           // This should match the filterable field of the
                          // corresponding Texture entry above.
                           ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                         count: None,
                    },
                  ],
                label: Some("texture_bind_group_layout"),
              });

    let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { x: (x as f32)*2.0 , y: (x as f32).sin()*10.0, z: (z as f32)*2.0 } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can affect scale if they're not created correctly
                    cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0)
                } else {
                    cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0)
                };

                Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
let instance_buffer = device.create_buffer_init(
    &wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX,
    }
);

let camera = Camera {
    position: (0.0, 10.0, 10.0).into(),

    // Looking roughly toward -Z
    yaw: -std::f32::consts::FRAC_PI_2,
    pitch: 0.0,

    aspect: config.width as f32 / config.height as f32,
    fovy: 75.0,
    znear: 0.05,
    zfar: 5000.0,
};

    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&camera);

    let camera_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );
    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    entries: &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    ],
    label: Some("camera_bind_group_layout"),
});
    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &camera_bind_group_layout,
    entries: &[
        wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }
    ],
    label: Some("camera_bind_group"),
});



    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Shader"),
    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
});

        let render_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            Some(&texture_bind_group_layout),
            Some(&camera_bind_group_layout),
        ],
            immediate_size: 0,
        }
    );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Some(model::ModelVertex::desc()), Some(InstanceRaw::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
                primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
            depth_stencil: Some(wgpu::DepthStencilState {
        format: texture::Texture::DEPTH_FORMAT,
        depth_write_enabled: Some(true),
        depth_compare: Some(wgpu::CompareFunction::Less), // 1.
        stencil: wgpu::StencilState::default(), // 2.
        bias: wgpu::DepthBiasState::default(),
    }),
        multisample: wgpu::MultisampleState {
            count: 1, // 2.
            mask: !0, // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        multiview_mask: None, // 5.
        cache: None, // 6.
        });

    let camera_controller = CameraController::new(
    5.0,
    0.0025,
);
    let obj_model =
    load_model("generic.obj", &device, &queue, &texture_bind_group_layout)
        .await
        .expect("load model failed lib:756");


        let mut world = world::World::new();

    world.generate_chunk(chunk::ChunkPos {
        x: 0,
        y: 0,
        z: 0,
    });

    let chunk = world
        .get_chunk(chunk::ChunkPos {
            x: 0,
            y: 0,
            z: 0,
        })
        .unwrap();

    let mesh = chunk_mesh::build_chunk_mesh(chunk);

    let chunk_mesh =
        chunk_mesh::GpuChunkMesh::new(&device, &mesh);

    Ok(Self {
        surface,
        device,
        queue,
        config,
        is_surface_configured: false,
        window,
        render_pipeline,
        camera,
        camera_uniform,
        camera_buffer,
        camera_bind_group,
        camera_controller,
        instances,
        instance_buffer,
        depth_texture,
        obj_model,
        world,
        chunk_mesh
    })



    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

pub fn tick(&mut self, dt: f32) {
    self.camera_controller
        .update_camera(&mut self.camera, dt);

    self.camera_uniform.update_view_proj(&self.camera);

    self.queue.write_buffer(
        &self.camera_buffer,
        0,
        bytemuck::cast_slice(&[self.camera_uniform]),
    );
}


    fn render(&mut self) -> anyhow::Result<()> {

        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // Skip this frame
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // You could recreate the devices and all resources
                // created with it here, but we'll just bail
                anyhow::bail!("Lost device");
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
            // This is what @location(0) in the fragment shader targets
            Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(
                        wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }
                    ),
                    store: wgpu::StoreOp::Store,
                }
            })
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &self.depth_texture.view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
    }),
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    
// lib.rs
render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

render_pass.set_pipeline(&self.render_pipeline);

for mesh in &self.obj_model.meshes {
    let material = &self.obj_model.materials[mesh.material];

    render_pass.draw_mesh_instanced(
        mesh,
        material,
        0..self.instances.len() as u32,
        &self.camera_bind_group,
    );
}
render_pass.set_vertex_buffer(
    0,
    self.chunk_mesh.vertex_buffer.slice(..),
);

render_pass.set_index_buffer(
    self.chunk_mesh.index_buffer.slice(..),
    wgpu::IndexFormat::Uint32,
);

render_pass.draw_indexed(
    0..self.chunk_mesh.num_indices,
    0,
    0..1,
);
    }
        self.queue.submit(iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }




    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera_controller.handle_key(code, is_pressed);
            
        }
    }
}

pub struct App {
    state: Option<State>,
    game_loop: main_loop::GameLoop,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            game_loop: main_loop::GameLoop::new(),
        }
    }
}

impl ApplicationHandler<State> for App {
    fn device_event(
    &mut self,
    _event_loop: &ActiveEventLoop,
    _device_id: winit::event::DeviceId,
    event: winit::event::DeviceEvent,
) {
    if let DeviceEvent::MouseMotion { delta } = event {
        if let Some(state) = &mut self.state {
            state.camera_controller.handle_mouse(
                delta.0 as f32,
                delta.1 as f32,
            );
        }
    }
}
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).expect("window failed to load lib:953"));
        window.set_cursor_visible(false);


        {
            // If we are not on web we can use pollster to
            // await the
            self.state = Some(pollster::block_on(State::new(window)).expect("failed to load window lib:964"));
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
    WindowEvent::CloseRequested => event_loop.exit(),

    WindowEvent::Resized(size) => {
        state.resize(size.width, size.height);
    }

    WindowEvent::RedrawRequested => {
        self.game_loop.update(state);

        match state.render() {
            Ok(_) => {}
            Err(e) => {
                log::error!("{e}");
                event_loop.exit();
            }
        }
    }

    WindowEvent::KeyboardInput {
        event:
            KeyEvent {
                physical_key: PhysicalKey::Code(code),
                state: key_state,
                ..
            },
        ..
    } => {
        state.handle_key(
            event_loop,
            code,
            key_state.is_pressed(),
        );
    }

    _ => {}
}
    }
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::new();
        event_loop.run_app(&mut app)?;
    }

    Ok(())
}