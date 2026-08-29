use anyhow::{anyhow, Result};
use glam::Vec3;
use noise::{NoiseFn};

use crate::{toroid::ToroidNoise};
// Voxel is a coordinate within a Chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl Voxel {
    fn as_usize(&self) -> usize {
        let x = (self.x as usize) & 0xFF;
        let y = (self.y as usize) & 0xFF;
        let z = (self.z as usize) & 0xFF;
        x + (y << 4) + (z << 8)
    }

    pub fn increment(v: u8) -> u8 {
        if v >= 15 {
            v
        } else {
            v + 1
        }
    }

    pub fn decrement(v: u8) -> u8 {
        if v == 0 {
            v
        } else {
            v - 1
        }
    }
}

impl From<usize> for Voxel {
    fn from(value: usize) -> Self {
        let x = value & 0xF;
        let y = (value >> 4) & 0xF;
        let z = (value >> 8) & 0xF;
        Voxel {
            x: x as u8,
            y: y as u8,
            z: z as u8,
        }
    }
}


pub const CHUNK_DIM: u8 = 16;
pub const CHUNK_DIMF: f32 = 16.;
pub const CHUNK_DIMF64 : f64 = 16.;

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk {
    pub dirty: bool,
    pub materials: Vec<Material>,
    pub version: u8,
}

impl Chunk {
    pub fn new(noise: &mut ToroidNoise, p: &ChunkCoord) -> Chunk {
        let mut materials =
            vec![Material::Air; CHUNK_DIM as usize * CHUNK_DIM as usize * CHUNK_DIM as usize];
        let mut set = false;
        for x in 0..CHUNK_DIM {
            for y in 0..CHUNK_DIM {
                for z in 0..CHUNK_DIM {
                    let voxel = Voxel { x, y, z };
                    let mat = sample_material(noise, &p, &voxel);
                    materials[voxel.as_usize()] = mat;
                    if mat != Material::Air {
                        set = true;
                    }
                }
            }
        }
        Chunk {
            dirty: set,
            materials,
            version: 0,
        }
    }
    pub fn update_voxel(&mut self, p: Voxel, m: Material) {
        self.dirty = true;
        self.materials[p.as_usize()] = m;
    }

    pub fn get_voxel(&self, p:Voxel) -> Material {
        self.materials[p.as_usize()]
    }
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Material {
    Air,
    Grass,
    Dirt,
    Diorite,
    Andesite,
    Magma,
}

impl Material {
    pub const fn top(&self) -> Option<f32> {
        match self {
            Material::Grass => Some(0.),
            _ => None,
        }
    }
    pub const fn bottom(&self) -> Option<f32> {
        match self {
            Material::Grass => Some(2.),
            _ => None,
        }
    }
    pub const fn sides(&self) -> f32 {
        match self {
            Material::Air => -1.,
            Material::Grass => 1.,
            Material::Dirt => 2.,
            Material::Diorite => 3.,
            Material::Andesite => 4.,
            Material::Magma => 5.,
        }
    }
}
const HALF_WORLD_HEIGHT: f64 = 8.;
fn sample_material(noise: &mut ToroidNoise, chunk: &ChunkCoord, p: &Voxel) -> Material {
    let x = (CHUNK_DIM as u32 * chunk.x + p.x as u32) as f64;
    let y = (CHUNK_DIM as u32 * chunk.y + p.y as u32) as f64;
    let z = (CHUNK_DIM as u32 * chunk.z + p.z as u32) as f64;
    if p.y == 0 && chunk.y == 0 {
        return Material::Magma;
    }
    let height = HALF_WORLD_HEIGHT + (noise.get([x, z]) * HALF_WORLD_HEIGHT).ceil();
    if y > height {
        return Material::Air;
    }

    if y + 0.01 > height || y == height {
        return Material::Grass;
    }
    if y > 0.75 * height {
        Material::Dirt
    } else {
        let n = noise.get([x, y, z]);
        if n < 0.5 {
            Material::Andesite
        } else {
            Material::Diorite
        }
    }
}

// ChunkCoord is the coordinate of a chunk in chunk units (see CHUNK_DIM)
#[derive(Debug)]
pub struct ChunkCoord {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl ChunkCoord {
    pub const ZERO: ChunkCoord = ChunkCoord { x: 0, y: 0, z: 0 };
    pub fn hash(&self) -> u64 {
        let x = (self.x as u64) & 0x1FFFFF;
        let y = (self.y as u64) & 0x1FFFFF;
        let z = (self.z as u64) & 0x1FFFFF;
        x + (y << 21) + (z << 42)
    }

    pub fn normalize(size: i32, x: i32, y: i32, z: i32) -> Result<ChunkCoord> {
        if x.abs() > size || y.abs() > size || z.abs() > size {
            Err(anyhow!("Dimension too large"))
        } else if y < 0 {
            Err(anyhow!("Y must be positive"))
        } else {
            Ok(ChunkCoord {
                x: wrap(x, size),
                y: y as u32,
                z: wrap(z, size),
            })
        }
    }

    pub fn distance_sq(&self, size: i32, other: &ChunkCoord) -> u64 {
        let mask = (size - 1) as u32;
        let half_world = size as u32 / 2;

        let dx = (self.x.wrapping_sub(other.x).wrapping_add(half_world) & mask) as i64 - half_world as i64;
        let dy = if self.y > other.y { self.y - other.y } else { other.y - self.y };
        let dz = (self.z.wrapping_sub(other.z).wrapping_add(half_world) & mask) as i64 - half_world as i64;

        let dx_64 = dx.abs() as u64;
        let dy_64 = dy as u64;
        let dz_64 = dz.abs() as u64;

        (dx_64 * dx_64) + (dy_64 * dy_64) + (dz_64 * dz_64)
    }
}

impl From<u64> for ChunkCoord {
    fn from(value: u64) -> Self {
        let x = value & 0x1FFFFF;
        let y = (value >> 21) & 0x1FFFFF;
        let z = (value >> 42) & 0x1FFFFF;
        ChunkCoord {
            x: x as u32,
            y: y as u32,
            z: z as u32,
        }
    }
}

fn wrap(x: i32, size: i32) -> u32 {
    if x < 0 {
        (size + x) as u32
    } else {
        x as u32
    }
}
