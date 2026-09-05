use anyhow::{Result};
use glam::{Vec3};
use nohash_hasher::NoHashHasher;
use rayon::{ThreadPool, ThreadPoolBuilder};
use sdl3::gpu::{
    BufferBinding, CommandBuffer, CopyPass, Device, RenderPass,
};
use std::{cmp::{Ordering, Reverse}, collections::{BinaryHeap, HashMap}, hash::BuildHasherDefault, sync::{Arc, mpsc}};

use crate::{chunk::{CHUNK_DIM, CHUNK_DIMF, CHUNK_DIMF64, Chunk, ChunkCoord}, chunk_render::{ChunkRender, Vertex, generate_mesh}, toroid::ToroidNoise};

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
    height_noise: Arc<ToroidNoise>,
    biome_noise: Arc<ToroidNoise>,
    size: i32,
    origin: ChunkCoord, // chunk the player is currently considered inside of
    chunks: HashMap<u64, Arc<Chunk>, BuildHasherDefault<NoHashHasher<u64>>>,
    chunk_renders: HashMap<u64, ChunkRender, BuildHasherDefault<NoHashHasher<u64>>>,
    chunks_to_load: BinaryHeap<Reverse<DistantChunk>>,
    pool: ThreadPool,
    chunk_tx: mpsc::Sender<(u64, Chunk)>,
    chunk_rx: mpsc::Receiver<(u64, Chunk)>,
    render_tx: mpsc::Sender<(u64, Vec<Vertex>)>,
    render_rx: mpsc::Receiver<(u64, Vec<Vertex>)>,
}

impl World {
    pub fn new(seed: u32, size: f64, threads: usize) -> Self {
        let (chunk_tx, chunk_rx) = mpsc::channel();
        let (render_tx, render_rx) = mpsc::channel();
        World {
            height_noise: Arc::new(ToroidNoise::new(seed, size * CHUNK_DIMF64)),
            biome_noise: Arc::new(ToroidNoise::new(seed+1, size * CHUNK_DIMF64)),
            size: size as i32,
            origin: ChunkCoord::ZERO,
            chunks: HashMap::with_hasher(BuildHasherDefault::default()),
            chunk_renders: HashMap::with_hasher(BuildHasherDefault::default()),
            chunks_to_load: BinaryHeap::new(),
            pool: ThreadPoolBuilder::new().num_threads(threads).build().unwrap(),
            chunk_tx,
            chunk_rx,
            render_tx,
            render_rx,
        }
    }
    pub fn load_chunk(&mut self, p: &ChunkCoord) {
        let tx = self.chunk_tx.clone();
        let height_noise = self.height_noise.clone();
        let p = *p;
        self.pool.spawn(move || {
            let chunk = Chunk::new(&height_noise, &p);
            let _ = tx.send((p.hash(), chunk));
        });
    }

    pub fn poll_chunks(&mut self) {
        while let Ok((hash, chunk)) = self.chunk_rx.try_recv() {
            let chunk = Arc::new(chunk);
            // todo handle versioning and origin relevance
            self.chunks.entry(hash).or_insert(chunk.clone());
            let chunk = chunk.clone();
            let tx = self.render_tx.clone();
            let biome_noise = self.biome_noise.clone();
            let coord = ChunkCoord::from(hash);
            self.pool.spawn(move || {
                if let Some(mesh) = generate_mesh(&chunk, coord, &biome_noise) {
                    let _ = tx.send((hash, mesh));
                }
            });
        }
    }

    pub fn poll_renders(&mut self, device: &Device, copy_pass: &CopyPass) -> Result<()> {
        while let Ok((hash, vertices)) = self.render_rx.try_recv() {
            let render = ChunkRender::upload(device, copy_pass, &vertices)?;
            self.chunk_renders.entry(hash).or_insert(render);
        }
        Ok(())
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