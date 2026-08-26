use anyhow::{Result};
use glam::Vec3;
use nohash_hasher::NoHashHasher;
use sdl3::gpu::{
    BufferBinding, CommandBuffer, CopyPass, Device, RenderPass,
};
use std::{collections::HashMap, hash::BuildHasherDefault};

use crate::{chunk::{CHUNK_DIM, CHUNK_DIMF, CHUNK_DIMF64, Chunk, ChunkCoord}, chunk_render::ChunkRender, toroid::ToroidNoise};


pub struct World {
    noise: ToroidNoise,
    size: i32,
    origin: ChunkCoord, // chunk the player is currently considered inside of
    chunks: HashMap<u64, Chunk, BuildHasherDefault<NoHashHasher<u64>>>,
    chunk_renders: HashMap<u64, ChunkRender, BuildHasherDefault<NoHashHasher<u64>>>
}

impl World {
    pub fn new(seed: u32, size: f64) -> Self {
        World {
            noise: ToroidNoise::new(seed, size * CHUNK_DIMF64),
            size: size as i32,
            origin: ChunkCoord::ZERO,
            chunks: HashMap::with_hasher(BuildHasherDefault::default()),
            chunk_renders: HashMap::with_hasher(BuildHasherDefault::default()),
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
        for (i, chunk) in &mut self.chunks {
            if limit == 0 {
                break;
            }
            if !chunk.dirty {
                continue;
            }
            let render = ChunkRender::generate_mesh(chunk, device, copy_pass)?;
            let _ = self.chunk_renders.entry(*i).insert_entry(render);
            limit -= 1;
            chunk.dirty = false;
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
                            if let Some(chunk) = self.chunk_renders.get(&c.hash()) {
                                    let binding = BufferBinding::default()
                                        .with_buffer(&chunk.buffer)
                                        .with_offset(0);
                                    let pos = Vec3::new(x as f32 * CHUNK_DIMF, y as f32 * CHUNK_DIMF,z as f32 * CHUNK_DIMF);
                                    command_buffer.push_vertex_uniform_data(1, &pos);
                                    render_pass.bind_vertex_buffers(0, &[binding]);
                                    render_pass.draw_indexed_primitives(chunk.indices as u32, 1, 0, 0, 0);
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
