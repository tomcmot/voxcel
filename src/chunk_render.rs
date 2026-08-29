use anyhow::Result;
use glam::Vec3;
use sdl3::gpu::{Buffer, BufferUsageFlags, CopyPass, Device, VertexAttribute, VertexBufferDescription, VertexElementFormat};

use crate::{chunk::{Chunk, Material, Voxel}, gpu_mem::upload_data};


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

fn vertices_at(direction: &Direction, position: Vec3, top: f32, bottom: f32, sides: f32) -> Vec<Vertex> {
    match direction {
        Direction::PosX => vec![
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
            
        ],
        Direction::NegX => vec![
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
        ],
        Direction::PosY => vec![
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
        ],
        Direction::NegY => vec![
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
        ],
        Direction::PosZ => vec![
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
        ],
        Direction::NegZ => vec![

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

        ],
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ
}

impl Direction {
    pub fn position_at(v: &Voxel, d: &Direction) -> Voxel {
        match d {
            Direction::PosX => Voxel {x: Voxel::increment(v.x), ..*v},
            Direction::NegX => Voxel {x: Voxel::decrement(v.x), ..*v},
            Direction::PosY => Voxel {y: Voxel::increment(v.y), ..*v},
            Direction::NegY => Voxel {y: Voxel::decrement(v.y), ..*v},
            Direction::PosZ => Voxel {z: Voxel::increment(v.z), ..*v},
            Direction::NegZ => Voxel {z: Voxel::decrement(v.z), ..*v},
        }
    }
    const ALL: [Direction; 6] = [
    Direction::PosX,
    Direction::NegX,
    Direction::PosY,
    Direction::NegY,
    Direction::PosZ,
    Direction::NegZ
    ];
}


pub struct ChunkRender {
    pub indices: usize,
    pub buffer: Buffer,
    pub version: u8,
}

impl ChunkRender {

    pub fn generate_mesh(
        chunk: &Chunk,
        device: &Device,
        copy_pass: &CopyPass,
    ) -> Result<Option<Self>> {
        let vertices = chunk
            .materials
            .iter()
            .enumerate()
            .map(|(index, mat)| {
                let material = *mat;
                if material == Material::Air {
                    return vec![];
                }
                let v = Voxel::from(index);
                let side = material.sides();
                let top = material.top().unwrap_or(side);
                let bottom = material.bottom().unwrap_or(side);
                Direction::ALL.iter().filter_map(|d| {
                    let v2 = Direction::position_at(&v, d);
                    let n = chunk.get_voxel(v2);
                    if n == Material::Air || v2 == v {
                        let mesh = vertices_at(d, Vec3::new(
                        v.x as f32,
                        v.y as f32,
                        v.z as f32,
                    ), top, bottom, side);
                        Some(mesh)
                    } else {
                        None
                    }
                }).collect::<Vec<Vec<Vertex>>>()
            })
            .flatten()
            .collect::<Vec<Vec<Vertex>>>()
            .concat();
        if vertices.len() == 0 {
            return Ok(None);
        }
        let buffer = device
            .create_buffer()
            .with_usage(BufferUsageFlags::VERTEX)
            .with_size((vertices.len() * size_of::<Vertex>()) as u32)
            .build()?;
        upload_data(device, copy_pass, &buffer, &vertices)?;
        Ok(Some(ChunkRender {
            indices: vertices.len() / 4 * 6,
            buffer,
            version: chunk.version
        }))
    }
}