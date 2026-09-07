use fun_wgpu::{
    wgpu,
};
use crate::anyhow::{
    Result,
    bail,
};
use bytemuck;

/// A change to a `LoadedWorld`
pub enum Change {
    Block {
        x: usize,
        y: usize,
        z: usize,
        value: u16
    },
    Cuboid {
        x: usize,
        y: usize,
        z: usize,
        width: usize,
        height: usize,
        length: usize,
        value: u16
    },
}

/// Loaded World Data
pub struct LoadedWorld {
    width: usize,
    length: usize,
    height: usize,
    data: Vec<u16>,
    // changes: Vec<(usize, usize, usize, usize, usize, usize, u16)>,
    texture: wgpu::Texture,
}

impl LoadedWorld {
    pub fn new(
        width: usize,
        length: usize,
        height: usize,
        data: Option<Vec<u16>>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<LoadedWorld> {
        let data = match data {
            Some(x) => {
                if x.len() != width * length * height {
                    bail!("Incorrect length of data!");
                }
                x
            },
            None => vec![0; width * length * height]
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("3D texture for the game world"),
            size: wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: length as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R16Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&data as &[u16]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((width * 2) as u32),
                rows_per_image: Some(height as u32),
            },
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: length as u32,
            }
        );
        Ok(LoadedWorld {
            width,
            length,
            height,
            data,
            // changes: Vec::new(),
            texture,
        })
    }
    #[inline]
    pub fn apply_change(
        &mut self,
        change: Change,
    ) {
        match change {
            Change::Block { x, y, z, value } => {
                self.data[z * self.width * self.height + y * self.width + x] = value;
            }
            Change::Cuboid { x, y, z, width, height, length, value } => {
                for z in z..(z+length) {
                    for y in y..(y+height) {
                        for x in x..(x+width) {
                            self.data[z * self.width * self.height + y * self.width + x] = value;
                        }
                    }
                }
            }
        }
    }
    pub fn update_texture(
        &self,
        queue: &wgpu::Queue,
    ) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&self.data as &[u16]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((self.width * 2) as u32),
                rows_per_image: Some(self.height as u32),
            },
            wgpu::Extent3d {
                width: self.width as u32,
                height: self.height as u32,
                depth_or_array_layers: self.length as u32,
            }
        );
    }
    pub fn create_view(&self) -> wgpu::TextureView {
        self.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        )
    }
}
