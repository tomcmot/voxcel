use anyhow::{anyhow, Result};
use glam::Vec3;
use noise::{NoiseFn};

use crate::{toroid::ToroidNoise};
// Voxel is a coordinate within a Chunk
#[derive(Debug)]
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
pub struct Chunk {
    pub dirty: bool,
    pub materials: Vec<Material>,
    pub version: u8,
}

impl Chunk {
    pub fn new(noise: &mut ToroidNoise, p: ChunkCoord) -> Chunk {
        let mut materials =
            vec![Material::Air; CHUNK_DIM as usize * CHUNK_DIM as usize * CHUNK_DIM as usize];
        for x in 0..CHUNK_DIM {
            for y in 0..CHUNK_DIM {
                for z in 0..CHUNK_DIM {
                    let voxel = Voxel { x, y, z };
                    materials[voxel.as_usize()] = sample_material(noise, &p, &voxel);
                }
            }
        }
        Chunk {
            dirty: true,
            materials,
            version: 0,
        }
    }
    pub fn update_voxel(&mut self, p: Voxel, m: Material) {
        self.dirty = true;
        self.materials[p.as_usize()] = m;
    }

}

#[repr(u16)]
#[derive(PartialEq, Eq, Copy, Clone)]
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

    pub fn distance(&self, size: f32, other: &ChunkCoord) -> f32 {
        let x = other.x as f32;
        let y = other.y as f32;
        let z = other.z as f32;
        let candidates = [
            Vec3::new(x,y,z),
            Vec3::new(x+size, y, z),
            Vec3::new(x, y, z + size),
            Vec3::new(x+size, y, z+size),
            Vec3::new(x-size, y, z),
            Vec3::new(x, y, z-size),
            Vec3::new(x-size, y, z-size),
            Vec3::new(x+size, y, z-size),
            Vec3::new(x-size, y, z+size),
        ];
        let me = Vec3::new(self.x as f32, self.y as f32, self.z as f32);
        
        candidates.iter().map(|c| c.distance(me)).min_by(f32::total_cmp).unwrap_or(f32::MAX)
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
