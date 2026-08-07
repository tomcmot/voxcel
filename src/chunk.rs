use glam::Vec3;
use noise::{NoiseFn, Simplex};
use sdl3::gpu::{VertexAttribute, VertexBufferDescription, VertexElementFormat};


#[repr(C)]
#[derive(Clone, Copy)]
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
#[rustfmt::skip]
pub const INDEXES: [u32; 36] = [
    // Top
    0, 1, 2,   2, 1, 3,
    // Bottom
    4, 6, 5,   6, 7, 5,
    // Front
    8, 10, 9,  10, 11, 9,
    // Back
    12, 13, 14, 14, 13, 15,
    // Left
    16, 18, 17, 18, 19, 17,
    // Right
    20, 21, 22, 22, 21, 23,
];

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indexes: Vec<u32>,
}

impl Mesh {
    pub fn new(position: Vec3, index: u32, top: f32, bottom: f32, sides: f32) -> Self {
        let vertices = vec![
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
        ];
        Mesh {
            vertices: vertices,
            indexes: INDEXES.map(|i| i + (index * 24)).to_vec(),
        }
    }
}

#[derive(Debug)]
pub struct Material {
    name: &'static str,
    // position in texture array
    top: Option<f32>,
    bottom: Option<f32>,
    sides: f32
}

const MATERIALS: [Material; 6] = [
    Material {
        name: "air",
        sides: -1.,
        top: None,
        bottom: None,
    },
    Material {
        name: "grass",
        top: Some(0.),
        bottom: Some(2.),
        sides: 1.
    },
    Material {
        name: "dirt",
        sides: 2.,
        top: None,
        bottom: None
    },
    Material {
        name: "diorite",
        sides: 3.,
        top: None,
        bottom: None
    },
    Material {
        name: "andesite",
        sides: 4.,
        top: None,
        bottom: None
    },
    Material {
        name: "magma",
        sides: 5.,
        top: None,
        bottom: None
    }
];

const CHUNK_DIM: usize = 16;

#[derive(Copy, Clone, Debug)]
pub struct Point {
    x: u32,
    y: u32,
    z: u32,
}
pub struct Chunk {
    materials: Vec<usize>, // dense array of material ids
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Chunk {
    pub fn new(seed: u64, pos: Vec3) -> Chunk {
        let noise = Simplex::new(seed as u32);
        let mut materials = vec![0; CHUNK_DIM * CHUNK_DIM * CHUNK_DIM];
        for x in 0 .. 16 {
            for y in 0 .. 16 {
                for z in 0 .. 16 {
                    let p = Point { x,y,z};
                    let material = sample_material(&p, &noise);
                    let i = encode_index(&p);
                    materials[i] = material;
                }
            }
        }
        let mut skipped = 0;
        let (vertices, indices) : (Vec<Vec<Vertex>>, Vec<Vec<u32>>)= materials.iter().enumerate().map(|(index, mat)| {
            if *mat == 0 {
                skipped += 1;
                return None;
            }
            let p = (CHUNK_DIM as f32 * pos) + decode_index(index);
            let material= &MATERIALS[*mat];
            let top = material.top.unwrap_or(material.sides);
            let bottom = material.bottom.unwrap_or(material.sides);
            let mesh = Mesh::new(p, (index - skipped) as u32, top, bottom, material.sides);
            Some((mesh.vertices, mesh.indexes))
        })
        .flatten()
        .collect::<Vec<(Vec<Vertex>, Vec<u32>)>>().into_iter().unzip();
        Chunk {
            materials,
            vertices: vertices.concat(),
            indices: indices.concat()
        }
    }
}

fn sample_material(pos: &Point,  noise: &Simplex) -> usize {
    if pos.y == 0 {
        5
    } else {
        let x = pos.x as f64 / 100.;
        let y = pos.y as f64 / 100.;
        let z = pos.z as f64 / 100.;
        let half_height = CHUNK_DIM as f64 / 2.;
        let height_map = (noise.get([x, z]) * half_height + half_height).ceil();
        if pos.y > height_map as u32 {
            0
        } else {
            let num_materials = 2.;
            let deviation = noise.get([x, y, z]) * 5. - 3.;
            let bands = (height_map / num_materials + deviation).ceil() as u32;
            if pos.y < bands {
                let mat = noise.get([x + 7., y + 3., z + 11.]);
                if mat < 0.5 {
                    4
                } else {
                    3
                }
            } else if pos.y == height_map as u32 {
                1
            } else {
                2
            }
        }
    }

}

// update this whenever CHUNK_SIZE changes
fn encode_index(p: &Point) -> usize {
    ((p.x & 0b1111) + ((p.y & 0b1111) << 4) + ((p.z & 0b1111) << 8)) as usize
}

fn decode_index(i: usize) -> Vec3 {
    let x = i & 0b1111;
    let y = (i >> 4) & 0b1111;
    let z = (i >> 8) & 0b1111;
    Vec3::new(x as f32, y as f32, z as f32)
}