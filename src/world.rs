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
    iwidth: i32,
    ilength: i32,
    iheight: i32,
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
            iwidth: width as i32,
            ilength: length as i32,
            iheight: height as i32,
            center_x: 0,
            center_y: 0,
            center_z: 0,
            data,
            // changes: Vec::new(),
            texture,
        })
    }
    #[inline]
    pub fn get_index_relative(&self, x: i32, y: i32, z: i32) -> usize {
        ((z + self.ilength/2) as usize) * self.width * self.height +
        ((y + self.iheight/2) as usize) * self.width +
        ((x + self.iwidth/2) as usize)
    }
    #[inline]
    pub fn get_index_basic(&self, x: i32, y: i32, z: i32) -> usize {
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
                if x < -self.iwidth/2 || x >= self.iwidth/2 {return;}
                if y < -self.iheight/2 || y >= self.iheight/2 {return;}
                if z < -self.ilength/2 || z >= self.ilength/2 {return;}
                let x1 = (x + self.iwidth/2) as usize;
                let y1 = (y + self.iheight/2) as usize;
                let z1 = (z + self.ilength/2) as usize;
                self.data[z1 * self.width * self.height + y1 * self.width + x1] = value;
            }
            Change::Cuboid { x, y, z, width, height, length, value } => {
                for i in 0..width {
                    for j in 0..height {
                        for k in 0..length {
                            self.apply_change_relative(Change::Block {
                                x: x + i,
                                y: y + j,
                                z: z + k,
                                value
                            });
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
                let x1 = (x - self.center_x + self.iwidth/2);
                let y1 = (y - self.center_y + self.iheight/2);
                let z1 = (z - self.center_z + self.ilength/2);
                if x1 < -self.iwidth/2 || x1 >= self.iwidth/2 {return;}
                if y1 < -self.iheight/2 || y1 >= self.iheight/2 {return;}
                if z1 < -self.ilength/2 || z1 >= self.ilength/2 {return;}
                let x1 = x1 as usize;
                let y1 = y1 as usize;
                let z1 = z1 as usize;
                if z1 < self.length && y1 < self.height {
                    if x1 < self.width {
                        self.data[z1 * self.width * self.height + y1 * self.width + x1] = value;
                    }
                }
            }
            Change::Cuboid { x, y, z, width, height, length, value } => {
                for i in 0..width {
                    for j in 0..height {
                        for k in 0..length {
                            self.apply_change_absolute(Change::Block {
                                x: x + i,
                                y: y + j,
                                z: z + k,
                                value
                            });
                        }
                    }
                }
            }
        }
    }
    pub fn translate_x(&mut self, amount: i32) {
        if amount == 0 {
            return;
        } else if amount < 0 {
            let amount = -amount;
            for a in 0..self.iheight {
                for b in 0..self.ilength {
                    for x in 0..(self.iwidth - amount) {
                        let ia = self.get_index_basic(x, a, b);
                        let ib = self.get_index_basic(x + amount, a, b);
                        self.data[ia] = self.data[ib];
                    }
                    for x in (self.iwidth - amount)..self.iwidth {
                        let ia = self.get_index_basic(x, a, b);
                        self.data[ia] = EMPTY;
                    }
                }
            }
        } else {
            for a in 0..self.iheight {
                for b in 0..self.ilength {
                    for x in (amount..self.iwidth).rev() {
                        let ia = self.get_index_basic(x, a, b);
                        let ib = self.get_index_basic(x - amount, a, b);
                        self.data[ia] = self.data[ib];
                    }
                    for x in 0..amount {
                        let ia = self.get_index_basic(x, a, b);
                        self.data[ia] = EMPTY;
                    }
                }
            }
        }
    }
    pub fn translate_y(&mut self, amount: i32) {
        if amount == 0 {
            return;
        } else if amount < 0 {
            let amount = -amount;
            for a in 0..self.iwidth {
                for b in 0..self.ilength {
                    for y in 0..(self.iheight - amount) {
                        let ia = self.get_index_basic(a, y, b);
                        let ib = self.get_index_basic(a, y + amount, b);
                        self.data[ia] = self.data[ib];
                    }
                    for y in (self.iheight - amount)..self.iheight {
                        let ia = self.get_index_basic(a, y, b);
                        self.data[ia] = EMPTY;
                    }
                }
            }
        } else {
            for a in 0..self.iwidth {
                for b in 0..self.ilength {
                    for y in (amount..self.iheight).rev() {
                        let ia = self.get_index_basic(a, y, b);
                        let ib = self.get_index_basic(a, y - amount, b);
                        self.data[ia] = self.data[ib];
                    }
                    for y in 0..amount {
                        let ia = self.get_index_basic(a, y, b);
                        self.data[ia] = EMPTY;
                    }
                }
            }
        }
    }
    pub fn translate_z(&mut self, amount: i32) {
        if amount == 0 {
            return;
        } else if amount < 0 {
            let amount = -amount;
            for a in 0..self.iwidth {
                for b in 0..self.iheight {
                    for z in 0..(self.ilength - amount) {
                        let ia = self.get_index_basic(a, b, z);
                        let ib = self.get_index_basic(a, b, z + amount);
                        self.data[ia] = self.data[ib];
                    }
                    for z in (self.ilength - amount)..self.ilength {
                        let ia = self.get_index_basic(a, b, z);
                        self.data[ia] = EMPTY;
                    }
                }
            }
        } else {
            for a in 0..self.iwidth {
                for b in 0..self.iheight {
                    for z in (amount..self.ilength).rev() {
                        let ia = self.get_index_basic(a, b, z);
                        let ib = self.get_index_basic(a, b, z - amount);
                        self.data[ia] = self.data[ib];
                    }
                    for z in 0..amount {
                        let ia = self.get_index_basic(a, b, z);
                        self.data[ia] = EMPTY;
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
