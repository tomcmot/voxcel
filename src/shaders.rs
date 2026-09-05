use anyhow::Result;
use std::{ffi::CStr, fs};

use sdl3::gpu::{Device, Shader, ShaderFormat, ShaderStage};

// ideally this should get queried from the shaders
pub struct ShaderDesc {
    entry_point: &'static CStr,
    samplers: u32,
    uniform_buffers: u32,
    storage_textures: u32,
    storage_buffers: u32,
}

pub const VERTEX_SHADER: ShaderDesc = ShaderDesc {
    entry_point: c"vertex",
    samplers: 0,
    uniform_buffers: 2,
    storage_buffers: 0,
    storage_textures: 0,
};

pub const FRAG_SHADER: ShaderDesc = ShaderDesc {
    entry_point: c"fragment",
    samplers: 3,
    uniform_buffers: 1,
    storage_buffers: 0,
    storage_textures: 0,
};

pub fn create_shaders(
    device: &Device,
    path: String,
    vert: ShaderDesc,
    frag: ShaderDesc,
) -> Result<(Shader, Shader)> {
    let code = fs::read(path)?;
    let vertex_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, code.as_slice(), ShaderStage::Vertex)
        .with_entrypoint(vert.entry_point)
        .with_samplers(vert.samplers)
        .with_uniform_buffers(vert.uniform_buffers)
        .with_storage_textures(vert.storage_textures)
        .with_storage_buffers(vert.storage_buffers)
        .build()?;
    let frag_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, code.as_slice(), ShaderStage::Fragment)
        .with_entrypoint(frag.entry_point)
        .with_samplers(frag.samplers)
        .with_uniform_buffers(frag.uniform_buffers)
        .with_storage_textures(frag.storage_textures)
        .with_storage_buffers(frag.storage_buffers)
        .build()?;
    Ok((vertex_shader, frag_shader))
}
