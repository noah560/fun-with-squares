use fun_wgpu::{
    wgpu,
};
use crate::anyhow::{
    Result,
    bail,
};
use bytemuck;

/// An empty block
const EMPTY: u16 = 0b0000_0000_0000_0_000;

/// A change to a `LoadedWorld`
pub enum Change {
    Block {
        x: i32,
        y: i32,
        z: i32,
        value: u16
    },
    Cuboid {
        x: i32,
        y: i32,
        z: i32,
        width: i32,
        height: i32,
        length: i32,
        value: u16
    },
}

/// Loaded World Data
pub struct LoadedWorld {
    // World dimensions
    width: usize,
    length: usize,
    height: usize,
    // Where the loaded area is centered
    center_x: i32,
    center_y: i32,
    center_z: i32,
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
            center_x: 0,
            center_y: 0,
            center_z: 0,
            data,
            // changes: Vec::new(),
            texture,
        })
    }
    #[inline]
    pub fn get_index_relative(x: i32, y: i32, z: i32) -> usize {
        ((z + (self.length as i32)/2) as usize) * self.width * self.height +
        ((y + (self.height as i32)/2) as usize) * self.width +
        ((x + (self.width as i32)/2) as usize)
    }
    #[inline]
    pub fn get_index_basic(x: i32, y: i32, z: i32) -> usize {
        (z as usize) * self.width * self.height +
        (y as usize) * self.width +
        (x as usize)
    }
    #[inline]
    pub fn apply_change_relative(
        &mut self,
        change: Change,
    ) {
        match change {
            Change::Block { x, y, z, value } => {
                let x1 = x as usize;
                let y1 = y as usize;
                let z1 = z as usize;
                if z1 < self.length && y1 < self.height {
                    if x1 < self.width {
                        self.data[z1 * self.width * self.height + y1 * self.width + x1] = value;
                    }
                }
            }
            Change::Cuboid { x, y, z, width, height, length, value } => {
                let x1 = x as usize;
                let y1 = y as usize;
                let z1 = z as usize;
                let width1 = width as usize;
                let height1 = height as usize;
                let length1 = length as usize;
                for z1 in z1..(z1+length1) {
                    for y1 in y1..(y1+height1) {
                        for x1 in x1..(x1+width1) {
                            if z1 < self.length && y1 < self.height {
                                if x1 < self.width {
                                    self.data[z1 * self.width * self.height + y1 * self.width + x1] = value;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[inline]
    pub fn apply_change_absolute(
        &mut self,
        change: Change
    ) {
        match change {
            Change::Block { x, y, z, value } => {
                let x1 = (x - self.center_x + (self.width as i32)/2) as usize;
                let y1 = (y - self.center_y + (self.height as i32)/2) as usize;
                let z1 = (z - self.center_z + (self.length as i32)/2) as usize;
                if z1 < self.length && y1 < self.height {
                    if x1 < self.width {
                        self.data[z1 * self.width * self.height + y1 * self.width + x1] = value;
                    }
                }
            }
            Change::Cuboid { x, y, z, width, height, length, value } => {
                let x1 = (x - self.center_x + (self.width as i32)/2) as usize;
                let y1 = (y - self.center_y + (self.height as i32)/2) as usize;
                let z1 = (z - self.center_z + (self.length as i32)/2) as usize;
                let width1 = width as usize;
                let height1 = height as usize;
                let length1 = length as usize;
                for z1 in z1..(z1+length1) {
                    for y1 in y1..(y1+height1) {
                        for x1 in x1..(x1+width1) {
                            if z1 < self.length && y1 < self.height {
                                if x1 < self.width {
                                    self.data[z1 * self.width * self.height + y1 * self.width + x1] = value;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn translate_x(amount: i32) {
        if amount == 0 {
            return;
        } else if amount < 0 {
            let amount = -amount;
            for a in 0..self.height {
                for b in 0..self.length {
                    for x in 0..(self.width - amount) {
                        self.data[self.get_index_basic(x, a, b)] = self.data[self.get_index_basic(x + amount, a, b)];
                    }
                    for x in (self.width - amount)..self.width {
                        self.data[self.get_index_basic(x, a, b)] = EMPTY;
                    }
                }
            }
        } else {
            for a in 0..self.height {
                for b in 0..self.length {
                    for x in (amount..self.width).rev() {
                        self.data[self.get_index_basic(x, a, b)] = self.data[self.get_index_basic(x - amount, a, b)];
                    }
                    for x in 0..amount {
                        self.data[self.get_index_basic(x, a, b)] = EMPTY;
                    }
                }
            }
        }
    }
    pub fn translate_y(amount: i32) {
        if amount == 0 {
            return;
        } else if amount < 0 {
            let amount = -amount;
            for a in 0..self.width {
                for b in 0..self.length {
                    for y in 0..(self.height - amount) {
                        self.data[self.get_index_basic(a, y, b)] = self.data[self.get_index_basic(a, y + amount, b)];
                    }
                    for y in (self.height - amount)..self.width {
                        self.data[self.get_index_basic(a, y, b)] = EMPTY;
                    }
                }
            }
        } else {
            for a in 0..self.width {
                for b in 0..self.length {
                    for y in (amount..self.height).rev() {
                        self.data[self.get_index_basic(a, y, b)] = self.data[self.get_index_basic(a, y - amount, b)];
                    }
                    for y in 0..amount {
                        self.data[self.get_index_basic(a, y, b)] = EMPTY;
                    }
                }
            }
        }
    }
    pub fn translate_z(amount: i32) {
        if amount == 0 {
            return;
        } else if amount < 0 {
            let amount = -amount;
            for a in 0..self.width {
                for b in 0..self.height {
                    for z in 0..(self.length - amount) {
                        self.data[self.get_index_basic(a, b, z)] = self.data[self.get_index_basic(a, b, z + amount)];
                    }
                    for z in (self.length - amount)..self.length {
                        self.data[self.get_index_basic(a, b, z)] = EMPTY;
                    }
                }
            }
        } else {
            for a in 0..self.width {
                for b in 0..self.height {
                    for z in (amount..self.length).rev() {
                        self.data[self.get_index_basic(a, b, z)] = self.data[self.get_index_basic(a, b, z - amount)];
                    }
                    for z in 0..amount {
                        self.data[self.get_index_basic(a, b, z)] = EMPTY;
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
