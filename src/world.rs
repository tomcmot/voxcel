use anyhow::{anyhow, Result};
use glam::Vec3;
use nohash_hasher::NoHashHasher;
use noise::{NoiseFn};
use sdl3::gpu::{
    Buffer, BufferBinding, BufferUsageFlags, CommandBuffer, CopyPass, Device, RenderPass, VertexAttribute, VertexBufferDescription, VertexElementFormat,
};
use std::{collections::HashMap, hash::BuildHasherDefault};

use crate::{gpu_mem::upload_data, toroid::ToroidNoise};

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

struct VertexBuffer {
    vertices: usize,
    buffer: Buffer,
}

const CHUNK_DIM: u8 = 16;
const CHUNK_DIMF: f32 = 16.;
const CHUNK_DIMF64 : f64 = 16.;
pub struct Chunk {
    dirty: bool,
    materials: Vec<Material>,
    vertex_buffer: Option<VertexBuffer>,
    version: u8,
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
            vertex_buffer: None,
            version: 0,
        }
    }
    pub fn update_voxel(&mut self, p: Voxel, m: Material) {
        self.dirty = true;
        self.materials[p.as_usize()] = m;
    }

    pub fn generate_mesh(
        &mut self,
        device: &Device,
        copy_pass: &CopyPass,
    ) -> Result<()> {
        if !self.dirty && self.vertex_buffer.is_some() {
            return Ok(());
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
                    Vec3::new(
                        v.x as f32,
                        v.y as f32,
                        v.z as f32,
                    ),
                    top,
                    bottom,
                    side,
                );
                Some(mesh)
            })
            .flatten()
            .collect::<Vec<Vec<Vertex>>>()
            .concat();
        let buffer = device
            .create_buffer()
            .with_usage(BufferUsageFlags::VERTEX)
            .with_size((vertices.len() * size_of::<Vertex>()) as u32)
            .build()?;
        upload_data(device, copy_pass, &buffer, &vertices)?;
        self.vertex_buffer = Some(VertexBuffer {
            vertices: vertices.len(),
            buffer,
        });
        self.version += 1;
        self.dirty = false;
        Ok(())
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

    fn normalize(size: i32, x: i32, y: i32, z: i32) -> Result<ChunkCoord> {
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

pub struct World {
    noise: ToroidNoise,
    size: i32,
    origin: ChunkCoord, // chunk the player is currently considered inside of
    chunks: HashMap<u64, Chunk, BuildHasherDefault<NoHashHasher<u64>>>,
}

impl World {
    pub fn new(seed: u32, size: f64) -> Self {
        World {
            noise: ToroidNoise::new(seed, size * CHUNK_DIMF64),
            size: size as i32,
            origin: ChunkCoord::ZERO,
            chunks: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }
    pub fn load_chunk(&mut self, x: i32, y: i32, z: i32) -> Result<()> {
        let p= ChunkCoord::normalize(self.size, x, y, z)?;
        self.chunks
            .entry(p.hash())
            .or_insert_with(|| Chunk::new(&mut self.noise, p));
        Ok(())
    }

    pub fn load_chunks(&mut self, distance: i32) -> Result<()> {
        for x in -distance..distance {
            for z in -distance..distance {
                self.load_chunk(self.origin.x as i32 + x, self.origin.y as i32, self.origin.z as i32 + z)?;
            }
        }
        Ok(())
    }

    pub fn update_origin(&mut self, o: ChunkCoord) {
        self.chunks
            .retain(|&k, _| ChunkCoord::from(k).distance(self.size as f32, &o) < 32.);
        self.origin = o;
    }

    pub fn generate(&mut self, device: &Device, copy_pass: &CopyPass) -> Result<()> {
        let mut limit = 4;
        // todo sort by distance to origin
        for (_i, chunk) in &mut self.chunks {
            if limit == 0 {
                break;
            }
            if !chunk.dirty {
                continue;
            }
            chunk.generate_mesh(device, copy_pass)?;
            limit -= 1;
        }
        Ok(())
    }

    pub fn render(&self, command_buffer: &CommandBuffer, render_pass: &RenderPass) {
        let render_distance = 16;
        for x in -render_distance..render_distance {
            for y in i32::max(self.origin.y as i32 - render_distance, 0)..render_distance {
                for z in -render_distance..render_distance {
                    match ChunkCoord::normalize(
                        self.size,
                        self.origin.x as i32 + x,
                        self.origin.y as i32 + y,
                        self.origin.z as i32 + z
                    ) {
                        Err(e) => { println!("{}", e)},
                        Ok(c) => {
                            if let Some(chunk) = self.chunks.get(&c.hash()) {
                                if let Some(buffer) = &chunk.vertex_buffer {
                                    let binding = BufferBinding::default()
                                        .with_buffer(&buffer.buffer)
                                        .with_offset(0);
                                    let pos = Vec3::new(x as f32 * CHUNK_DIMF, y as f32 * CHUNK_DIMF,z as f32 * CHUNK_DIMF);
                                    command_buffer.push_vertex_uniform_data(1, &pos);
                                    render_pass.bind_vertex_buffers(0, &[binding]);
                                    render_pass.draw_indexed_primitives((buffer.vertices / 4 * 6) as u32, 1, 0, 0, 0);
                                }
                            }
                        }
                    }
                }
            }
        }
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
