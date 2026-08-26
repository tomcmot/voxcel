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
    ) -> Result<Self> {
        let mut skipped = 0;
        let vertices = chunk
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
        Ok(ChunkRender {
            indices: vertices.len() / 4 * 6,
            buffer,
            version: chunk.version
        })
    }
}