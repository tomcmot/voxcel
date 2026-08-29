use anyhow::{Result};
use glam::Vec3;
use nohash_hasher::NoHashHasher;
use sdl3::gpu::{
    BufferBinding, CommandBuffer, CopyPass, Device, RenderPass,
};
use std::{cmp::{Ordering, Reverse}, collections::{BinaryHeap, HashMap}, hash::BuildHasherDefault};

use crate::{chunk::{CHUNK_DIM, CHUNK_DIMF, CHUNK_DIMF64, Chunk, ChunkCoord}, chunk_render::ChunkRender, toroid::ToroidNoise};

#[derive(Debug, Eq, PartialEq)]
struct DistantChunk {
    distance_sq: u64,
    chunk: u64
}

impl PartialOrd for DistantChunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance_sq.partial_cmp(&other.distance_sq)
    }
}

impl Ord for DistantChunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance_sq.cmp(&other.distance_sq)
    }
}

pub struct World {
    noise: ToroidNoise,
    size: i32,
    origin: ChunkCoord, // chunk the player is currently considered inside of
    chunks: HashMap<u64, Chunk, BuildHasherDefault<NoHashHasher<u64>>>,
    chunk_renders: HashMap<u64, ChunkRender, BuildHasherDefault<NoHashHasher<u64>>>,
    chunks_to_generate: BinaryHeap<Reverse<DistantChunk>>,
    chunks_to_load: BinaryHeap<Reverse<DistantChunk>>,
}

impl World {
    pub fn new(seed: u32, size: f64) -> Self {
        World {
            noise: ToroidNoise::new(seed, size * CHUNK_DIMF64),
            size: size as i32,
            origin: ChunkCoord::ZERO,
            chunks: HashMap::with_hasher(BuildHasherDefault::default()),
            chunk_renders: HashMap::with_hasher(BuildHasherDefault::default()),
            chunks_to_generate: BinaryHeap::new(),
            chunks_to_load: BinaryHeap::new()
        }
    }
    pub fn load_chunk(&mut self, p: &ChunkCoord) {
        let chunk = Chunk::new(&mut self.noise, &p);
        if chunk.dirty {
            self.chunks_to_generate.push(Reverse(DistantChunk { distance_sq: self.origin.distance_sq(self.size, &p), chunk: p.hash() }))
        }
        self.chunks
            .entry(p.hash())
            .or_insert_with(|| chunk);
    }

    pub fn load_chunks(&mut self) {
        let mut limit = 4;
        while limit > 0 && let Some(Reverse(chunk)) = self.chunks_to_load.pop() {
            let coord = ChunkCoord::from(chunk.chunk);
            self.load_chunk(&coord);
            limit -= 1;
        }
    }

    pub fn queue_chunks(&mut self, distance: i32) -> Result<()> {
        for x in -distance..distance {
            for y in i32::max(0, self.origin.y as i32 - distance)..(self.origin.y as i32 + distance) {
                for z in -distance..distance {
                    let coord = ChunkCoord::normalize(self.size, self.origin.x as i32 + x, self.origin.y as i32 + y, self.origin.z as i32 + z)?;
                    let distance_sq = self.origin.distance_sq(self.size, &coord);
                    self.chunks_to_load.push(Reverse(DistantChunk { distance_sq, chunk: coord.hash() }))
                }
            }
        }
        Ok(())
    }

    pub fn update_origin(&mut self, o: ChunkCoord) {
        self.chunks
            .retain(|&k, _| ChunkCoord::from(k).distance_sq(self.size, &o) < 32*32);
        self.origin = o;
    }

    pub fn generate(&mut self, device: &Device, copy_pass: &CopyPass) -> Result<()> {
        let mut limit = 4;
        while limit > 0 && let Some(Reverse(distant_chunk)) = self.chunks_to_generate.pop() {
            if let Some(chunk) = self.chunks.get_mut(&distant_chunk.chunk) {

                let render = ChunkRender::generate_mesh(chunk, device, copy_pass)?;
                match render {
                    None => {},
                    Some(mesh) => {
                        let _ = self.chunk_renders.entry(distant_chunk.chunk).insert_entry(mesh);
                    },
                }
                limit -= 1;
                chunk.dirty = false;
            }

        }
        Ok(())
    }

    pub fn render(&self, command_buffer: &CommandBuffer, render_pass: &RenderPass) {
        for (u, chunk) in &self.chunk_renders {
            let coord = ChunkCoord::from(*u);
            let pos = canon_to_local(&coord, &self.origin, self.size);
            let binding = BufferBinding::default()
                    .with_buffer(&chunk.buffer)
                    .with_offset(0);
            command_buffer.push_vertex_uniform_data(1, &pos);
            render_pass.bind_vertex_buffers(0, &[binding]);
            render_pass.draw_indexed_primitives(chunk.indices as u32, 1, 0, 0, 0);
        }
    }

    pub fn worst_case_indexes() -> Vec<u16> {
        let voxels_per_chunk = CHUNK_DIM as u16 * CHUNK_DIM as u16 * CHUNK_DIM as u16 / 2;
        // voxels_per_chunk * cube faces * indexes per face (aka 2 triangles of 3 indexes)
        let max_indices = (voxels_per_chunk as usize * 6 * 6) as usize;
        let mut indices = vec![0; max_indices];
        let faces = (voxels_per_chunk * 6) as usize;
        for i in 0..faces {
            // pattern per face
            // 0, 1, 2, 2, 1, 3,
            indices[i * 6] = (i * 4) as u16;
            indices[i * 6 + 1] = (i * 4 + 1) as u16;
            indices[i * 6 + 2] = (i * 4 + 2) as u16;
            indices[i * 6 + 3] = (i * 4 + 2) as u16;
            indices[i * 6 + 4] = (i * 4 + 1) as u16;
            indices[i * 6 + 5] = (i * 4 + 3) as u16;
        }
        indices
    }
}

fn canon_to_local(coord: &ChunkCoord, origin: &ChunkCoord, size: i32) -> Vec3 {
    let dx = wrap(coord.x as i32 - origin.x as i32, size);
    let dz = wrap(coord.z as i32 - origin.z as i32, size);
    let dy = coord.y as i32 - origin.y as i32;
    Vec3::new(dx as f32 * CHUNK_DIMF,dy as f32 * CHUNK_DIMF, dz as f32 * CHUNK_DIMF)
}

fn wrap(v: i32, size: i32) -> i32 {
    if v > size / 2 { 
        v - size 
    } else if v < -size / 2 { 
        v + size 
    } else { 
        v 
    }
}