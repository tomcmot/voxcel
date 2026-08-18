use std::{collections::HashMap, hash::BuildHasherDefault};

use glam::{Mat4, Vec3};
use nohash_hasher::NoHashHasher;
use noise::{NoiseFn, Simplex};
use sdl3::gpu::{VertexAttribute, VertexBufferDescription, VertexElementFormat};

// Voxel is a coordinate within a Chunk
#[derive(Debug)]
pub struct Voxel {
    x: u8,
    y: u8,
    z: u8,
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uvw: Vec3,
}

impl Vertex {
    pub fn buffer_desc() -> VertexBufferDescription {
        VertexBufferDescription::default()
            .with_slot(0)
            .with_input_rate(sdl3::gpu::VertexInputRate::Vertex)
            .with_instance_step_rate(0)
            .with_pitch(size_of::<Vertex>() as u32)
    }

    pub fn attributes() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::default()
                .with_buffer_slot(0)
                .with_location(0)
                .with_format(VertexElementFormat::Float3)
                .with_offset(0),
            VertexAttribute::default()
                .with_buffer_slot(0)
                .with_location(1)
                .with_format(VertexElementFormat::Float3)
                .with_offset((size_of::<f32>() * 3) as u32),
            VertexAttribute::default()
                .with_buffer_slot(0)
                .with_location(2)
                .with_format(VertexElementFormat::Float3)
                .with_offset((size_of::<f32>() * 6) as u32),
        ]
    }
}

fn new_cube_vertices(position: Vec3, top: f32, bottom: f32, sides: f32) -> Vec<Vertex> {
    vec![
        Vertex {
            position: Vec3::new(-0.5, 0.5, 0.5) + position,
            normal: Vec3::new(0., 1., 0.),
            uvw: Vec3::new(0., 0., top),
        }, // Top Left Front
        Vertex {
            position: Vec3::new(0.5, 0.5, 0.5) + position,
            normal: Vec3::new(0., 1., 0.),
            uvw: Vec3::new(1., 0., top),
        }, // Top Right Front
        Vertex {
            position: Vec3::new(-0.5, 0.5, -0.5) + position,
            normal: Vec3::new(0., 1., 0.),
            uvw: Vec3::new(0., 1., top),
        }, // Top Left Back
        Vertex {
            position: Vec3::new(0.5, 0.5, -0.5) + position,
            normal: Vec3::new(0., 1., 0.),
            uvw: Vec3::new(1., 1., top),
        }, // Top Right Back
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.5) + position,
            normal: Vec3::new(0., -1., 0.),
            uvw: Vec3::new(0., 0., bottom),
        }, // Bottom Left Front
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.5) + position,
            normal: Vec3::new(0., -1., 0.),
            uvw: Vec3::new(1., 0., bottom),
        }, // Bottom Right Front
        Vertex {
            position: Vec3::new(-0.5, -0.5, -0.5) + position,
            normal: Vec3::new(0., -1., 0.),
            uvw: Vec3::new(0., 1., bottom),
        }, // Bottom Left Back
        Vertex {
            position: Vec3::new(0.5, -0.5, -0.5) + position,
            normal: Vec3::new(0., -1., 0.),
            uvw: Vec3::new(1., 1., bottom),
        }, // Bottom Right Back
        Vertex {
            position: Vec3::new(-0.5, 0.5, 0.5) + position,
            normal: Vec3::new(0., 0., 1.),
            uvw: Vec3::new(0., 0., sides),
        }, // Top Left Front
        Vertex {
            position: Vec3::new(0.5, 0.5, 0.5) + position,
            normal: Vec3::new(0., 0., 1.),
            uvw: Vec3::new(1., 0., sides),
        }, // Top Right Front
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.5) + position,
            normal: Vec3::new(0., 0., 1.),
            uvw: Vec3::new(0., 1., sides),
        }, // Bottom Left Front
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.5) + position,
            normal: Vec3::new(0., 0., 1.),
            uvw: Vec3::new(1., 1., sides),
        }, // Bottom Right Front
        Vertex {
            position: Vec3::new(-0.5, 0.5, -0.5) + position,
            normal: Vec3::new(0., 0., -1.),
            uvw: Vec3::new(1., 0., sides),
        }, // Top Left Back
        Vertex {
            position: Vec3::new(0.5, 0.5, -0.5) + position,
            normal: Vec3::new(0., 0., -1.),
            uvw: Vec3::new(0., 0., sides),
        }, // Top Right Back
        Vertex {
            position: Vec3::new(-0.5, -0.5, -0.5) + position,
            normal: Vec3::new(0., 0., -1.),
            uvw: Vec3::new(1., 1., sides),
        }, // Bottom Left Back
        Vertex {
            position: Vec3::new(0.5, -0.5, -0.5) + position,
            normal: Vec3::new(0., 0., -1.),
            uvw: Vec3::new(0., 1., sides),
        }, // Bottom Right Back
        Vertex {
            position: Vec3::new(-0.5, 0.5, 0.5) + position,
            normal: Vec3::new(-1., 0., 0.),
            uvw: Vec3::new(1., 0., sides),
        }, // Top Left Front
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.5) + position,
            normal: Vec3::new(-1., 0., 0.),
            uvw: Vec3::new(1., 1., sides),
        }, // Bottom Left Front
        Vertex {
            position: Vec3::new(-0.5, 0.5, -0.5) + position,
            normal: Vec3::new(-1., 0., 0.),
            uvw: Vec3::new(0., 0., sides),
        }, // Top Left Back
        Vertex {
            position: Vec3::new(-0.5, -0.5, -0.5) + position,
            normal: Vec3::new(-1., 0., 0.),
            uvw: Vec3::new(0., 1., sides),
        }, // Bottom Left Back
        Vertex {
            position: Vec3::new(0.5, 0.5, 0.5) + position,
            normal: Vec3::new(1., 0., 0.),
            uvw: Vec3::new(0., 0., sides),
        }, // Top Right Front
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.5) + position,
            normal: Vec3::new(1., 0., 0.),
            uvw: Vec3::new(0., 1., sides),
        }, // Bottom Right Front
        Vertex {
            position: Vec3::new(0.5, 0.5, -0.5) + position,
            normal: Vec3::new(1., 0., 0.),
            uvw: Vec3::new(1., 0., sides),
        }, // Top Right Back
        Vertex {
            position: Vec3::new(0.5, -0.5, -0.5) + position,
            normal: Vec3::new(1., 0., 0.),
            uvw: Vec3::new(1., 1., sides),
        }, // Bottom Right Back
    ]
}

const CHUNK_DIM: u8 = 16;
const CHUNK_DIMF: f32 = 16.;
pub struct Chunk {
    dirty: bool,
    materials: Vec<Material>,
    vertices: Vec<Vertex>,
    version: u8,
}

impl Chunk {
    pub fn new(noise: &mut Simplex, p: ChunkCoord) -> Chunk {
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
            vertices: vec![],
            version: 0,
        }
    }
    pub fn update_voxel(&mut self, p: Voxel, m: Material) {
        self.dirty = true;
        self.materials[p.as_usize()] = m;
    }

    pub fn generate_mesh(&mut self) {
        if !self.dirty {
            return;
        }
        let mut skipped = 0;
        let vertices = self
            .materials
            .iter()
            .enumerate()
            .map(|(index, mat)| {
                let material = *mat;
                if material == Material::Air {
                    skipped += 1;
                    return None;
                }
                let v = Voxel::from(index);
                let side = material.sides();
                let top = material.top().unwrap_or(side);
                let bottom = material.bottom().unwrap_or(side);
                let mesh = new_cube_vertices(
                    Vec3::new(v.x as f32, v.y as f32, v.z as f32),
                    top,
                    bottom,
                    side,
                );
                Some(mesh)
            })
            .flatten()
            .collect::<Vec<Vec<Vertex>>>();
        self.vertices = vertices.concat();
        self.version += 1;
        self.dirty = false;
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
    const fn top(&self) -> Option<f32> {
        match self {
            Material::Grass => Some(0.),
            _ => None,
        }
    }
    const fn bottom(&self) -> Option<f32> {
        match self {
            Material::Grass => Some(2.),
            _ => None,
        }
    }
    const fn sides(&self) -> f32 {
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
fn sample_material(noise: &mut Simplex, chunk: &ChunkCoord, p: &Voxel) -> Material {
    let x = (CHUNK_DIM as u32 * chunk.x + p.x as u32) as f64;
    let y = (CHUNK_DIM as u32 * chunk.y + p.y as u32) as f64;
    let z = (CHUNK_DIM as u32 * chunk.z + p.z as u32) as f64;
    let sample_x = x * 0.01;
    let sample_y = y * 0.01;
    let sample_z = z * 0.01;
    if p.y == 0 && chunk.y == 0 {
        return Material::Magma;
    }
    let height = HALF_WORLD_HEIGHT + (noise.get([sample_x, sample_z]) * HALF_WORLD_HEIGHT).ceil();
    if y > height {
        return Material::Air;
    }

    if y + 0.01 > height || y == height {
        return Material::Grass;
    }
    if y > 0.75 * height {
        Material::Dirt
    } else {
        let n = noise.get([sample_x, sample_y, sample_z]);
        if n < 0.5 {
            Material::Andesite
        } else {
            Material::Diorite
        }
    }
}

// ChunkCoord is the coordinate of a chunk in chunk units (see CHUNK_DIM)
pub struct ChunkCoord {
    x: u32,
    y: u32,
    z: u32,
}

impl ChunkCoord {
    pub const ZERO: ChunkCoord = ChunkCoord { x: 0, y: 0, z: 0 };
    pub fn hash(&self) -> u64 {
        let x = (self.x as u64) & 0x1FFFFF;
        let y = (self.y as u64) & 0x1FFFFF;
        let z = (self.z as u64) & 0x1FFFFF;
        x + (y << 21) + (z << 42)
    }

    pub fn distance(&self, other: &ChunkCoord) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
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

pub struct World {
    noise: Simplex,
    origin: ChunkCoord, // chunk the player is currently considered inside of
    chunks: HashMap<u64, Chunk, BuildHasherDefault<NoHashHasher<u64>>>,
}

impl World {
    pub fn new(seed: u32) -> Self {
        World {
            noise: Simplex::new(seed),
            origin: ChunkCoord::ZERO,
            chunks: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }
    pub fn load_chunk(&mut self, p: ChunkCoord) {
        self.chunks
            .entry(p.hash())
            .or_insert_with(|| Chunk::new(&mut self.noise, p));
    }

    pub fn update_origin(&mut self, o: ChunkCoord) {
        self.chunks
            .retain(|&k, _| ChunkCoord::from(k).distance(&o) < 32.);
        self.origin = o;
    }

    pub fn generate(&mut self) {
        let mut limit = 4;
        for (_, chunk) in &mut self.chunks {
            if limit == 0 {
                break;
            }
            if !chunk.dirty {
                continue;
            }
            chunk.generate_mesh();
            limit -= 1;
        }
    }

    pub fn render(&self) -> Vec<(Mat4, u8, &Vec<Vertex>)> {
        self.chunks
            .iter()
            .map(|(index, chunk)| {
                let coord = ChunkCoord::from(*index);
                let translate = Mat4::from_translation(Vec3::new(
                    coord.x as f32 * CHUNK_DIMF,
                    coord.y as f32 * CHUNK_DIMF,
                    coord.z as f32 * CHUNK_DIMF,
                ));
                (translate, chunk.version, &chunk.vertices)
            })
            .collect()
    }
    pub fn worst_case_indexes() -> Vec<u32> {
        let voxels_per_chunk = CHUNK_DIM as u32 * CHUNK_DIM as u32 * CHUNK_DIM as u32;
        // voxels_per_chunk * cube faces * indexes per face (aka 2 triangles of 3 indexes)
        let max_indices = (voxels_per_chunk * 6 * 6) as usize;
        let mut indices = vec![0; max_indices];
        let faces = (voxels_per_chunk * 6) as usize;
        for i in 0..faces {
            // pattern per face
            // 0, 1, 2, 2, 1, 3,
            indices[i * 6] = (i * 4) as u32;
            indices[i * 6 + 1] = (i * 4 + 1) as u32;
            indices[i * 6 + 2] = (i * 4 + 2) as u32;
            indices[i * 6 + 3] = (i * 4 + 2) as u32;
            indices[i * 6 + 4] = (i * 4 + 1) as u32;
            indices[i * 6 + 5] = (i * 4 + 3) as u32;
        }
        indices
    }
}
