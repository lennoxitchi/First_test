use anyhow::{anyhow, Result};
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TextureAtlas {
    pub texture: Texture,

    /// Texture name -> tile index
    pub indices: HashMap<String, u32>,

    pub tile_size: u32,
    pub columns: u32,
    pub rows: u32,
}

impl TextureAtlas {
    pub fn build(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        tile_size: u32,
    ) -> Result<Self> {
        // These are embedded into the executable at compile time.
        let textures: &[(&str, &[u8])] = &[
            ("stone", include_bytes!("res/assets/stone.png")),
            ("dirt", include_bytes!("res/assets/dirt.png")),
            ("grass_side", include_bytes!("res/assets/grass_side.png"),  ),
            ("grass_top", include_bytes!("res/assets/grass_top.png"),  ),
        ];

        if textures.is_empty() {
            return Err(anyhow!("No textures defined"));
        }

        let mut images = Vec::with_capacity(textures.len());

        for (name, bytes) in textures {
            let image = image::load_from_memory(bytes)?;

            if image.width() != tile_size
                || image.height() != tile_size
            {
                return Err(anyhow!(
                    "Texture '{}' is {}x{}, expected {}x{}",
                    name,
                    image.width(),
                    image.height(),
                    tile_size,
                    tile_size
                ));
            }

            images.push(image.to_rgba8());
        }

        let texture_count = images.len() as u32;

        let columns =
            (texture_count as f32).sqrt().ceil() as u32;

        let rows =
            (texture_count + columns - 1) / columns;

        let atlas_width = columns * tile_size;
        let atlas_height = rows * tile_size;

        let mut atlas =
            RgbaImage::new(atlas_width, atlas_height);

        let mut indices = HashMap::new();

        for (index, ((name, _), image)) in
            textures.iter().zip(images.iter()).enumerate()
        {
            let index = index as u32;

            let x = (index % columns) * tile_size;
            let y = (index / columns) * tile_size;

            image::imageops::replace(
                &mut atlas,
                image,
                x as i64,
                y as i64,
            );

            indices.insert((*name).to_string(), index);
        }

        let dynamic =
            DynamicImage::ImageRgba8(atlas);

        let texture = Texture::from_image(
            device,
            queue,
            &dynamic,
            Some("Block Texture Atlas"),
        )?;

        Ok(Self {
            texture,
            indices,
            tile_size,
            columns,
            rows,
        })
    }

    pub fn index(&self, name: &str) -> Result<u32> {
        self.indices
            .get(name)
            .copied()
            .ok_or_else(|| {
                anyhow!("Texture '{}' does not exist", name)
            })
    }
}

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.
    
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8], 
        label: &str
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,

                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,

                ..Default::default()
            }
        );

        Ok(Self { texture, view, sampler })
    }

    
}

pub fn create_white_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Texture {
    let rgba = [255u8, 255u8, 255u8, 255u8];

    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("White Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("White Texture Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    Texture {
        texture,
        view,
        sampler,
    }
}