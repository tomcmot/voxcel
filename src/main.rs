use std::{ffi::CStr, fs};

use anyhow::Result;
use glam::{
    Mat4, Vec2, Vec3, camera::rh::{proj::directx, view::look_to_mat4},
};
use sdl3::{
    event::Event,
    gpu::{
        BlendFactor, BlendOp, BufferBinding, BufferRegion, BufferUsageFlags, ColorTargetBlendState,
        ColorTargetDescription, ColorTargetInfo, CommandBuffer, CompareOp, CopyPass, CullMode,
        DepthStencilState, DepthStencilTargetInfo, Device, GraphicsPipeline,
        GraphicsPipelineTargetInfo, IndexElementSize, LoadOp, PrimitiveType, RasterizerState,
        RenderPass, SampleCount, Sampler, SamplerCreateInfo, Shader, ShaderFormat, ShaderStage,
        StoreOp, Texture, TextureCreateInfo, TextureFormat, TextureRegion, TextureSamplerBinding,
        TextureTransferInfo, TextureType, TextureUsage, TransferBufferLocation,
        TransferBufferUsage, VertexInputState,
    },
    keyboard::{Keycode, Scancode},
    pixels::Color,
    sys::timer::SDL_GetTicksNS,
    video::Window,
};

mod chunk;
mod ui;

use chunk::Vertex;

use crate::chunk::Chunk;

#[repr(C)]
#[derive(Clone, Copy)]
struct Camera {
    position: Vec3,
    front: Vec3,
    up: Vec3,
    pitch: f32,
    yaw: f32,
    zoom: f32,
}

#[repr(C)]
#[derive(Debug)]
struct CameraBuffer {
    proj_view: Mat4,
    model: Mat4,
    normal: Mat4,
}

#[repr(C)]
struct LightingBuffer {
    ambient_color: Vec3,
    band_count: f32,
    light_dir: Vec3,
    _pad0: f32,
    light_color: Vec3,
}

#[repr(C)]
struct OutlineBuffer {
    color: Vec3,
    thickness: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::new(0., 0., 5.),
            front: -Vec3::Z,
            up: Vec3::Y,
            pitch: 0.,
            yaw: -90.,
            zoom: 45.,
        }
    }
}

const INVERT_Y: bool = false;
impl Camera {
    fn move_to(&mut self, p: Vec3) {
        self.position = p;
    }

    fn view(&self) -> Mat4 {
        look_to_mat4(self.position, self.front, self.up)
    }

    /// SDL3 GPU uses the directx Z convention
    fn projection(&self, width: f32, height: f32) -> Mat4 {
        directx::perspective(self.zoom.to_radians(), width / height, 0.01, 200.)
    }

    fn rotate(&mut self, x_offset: f32, y_offset: f32) {
        self.yaw += x_offset;
        self.pitch = (self.pitch - y_offset).clamp(-89.0, 89.0);
        // save the radians conversions
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        // calculate new facing
        self.front = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            if INVERT_Y {
                -pitch_rad.sin()
            } else {
                pitch_rad.sin()
            },
            yaw_rad.sin() * pitch_rad.cos(),
        )
        .normalize();
    }
}

// todo these constants should be swapped to be queried at runtime
const SHADER_PATH: &'static str = "assets/";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const TEXTURE_PATH: &'static str = "assets/blocks.png";
fn main() -> Result<()> {
    let _ = sdl3::hint::set(sdl3::hint::names::RENDER_VULKAN_DEBUG, "1");
    let mut sdl = sdl3::init()?;
    let video = sdl.video()?;
    let mut window = video
        .window("voxcell", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .vulkan()
        .build()?;
    window.set_mouse_grab(true);
    sdl.mouse().set_relative_mouse_mode(&window, true);
    let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;
    device.set_swapchain_parameters(&window, sdl3::gpu::PresentMode::Immediate, sdl3::gpu::SwapchainComposition::Sdr)?;
    let mut ui = ui::UI::new(&device, &window);
    let chunk = Chunk::new(0, Vec3::ZERO);
    let vertex_buffer = device
        .create_buffer()
        .with_size((chunk.vertices.len() * size_of::<Vertex>()) as u32)
        .with_usage(BufferUsageFlags::VERTEX)
        .build()?;
    let bindings = [BufferBinding::default()
        .with_buffer(&vertex_buffer)
        .with_offset(0)];
    let index_buffer = device
        .create_buffer()
        .with_size((chunk.indices.len() * size_of::<u32>()) as u32)
        .with_usage(BufferUsageFlags::INDEX)
        .build()?;
    let index_binding = BufferBinding::default()
        .with_buffer(&index_buffer)
        .with_offset(0);
    let (texture, sampler) = create_texture_sampler(&device)?;
    {
        let copy_commands = device.acquire_command_buffer()?;
        let copy_pass = device.begin_copy_pass(&copy_commands)?;
        upload_data(&device, &copy_pass, &vertex_buffer, &chunk.vertices)?;
        upload_data(&device, &copy_pass, &index_buffer, &chunk.indices)?;
        upload_texture(&device, &copy_pass, TEXTURE_PATH, &texture)?;
        device.end_copy_pass(copy_pass);
        let _ = copy_commands.submit()?;
    };

    let tex_samp_bind = [TextureSamplerBinding::default()
        .with_sampler(&sampler)
        .with_texture(&texture)];
    let (_depth_texture, depth_info) = create_depth_texture(&device)?;
    let pipeline = Renderer::new(&window, &device, SHADER_PATH.into())?;
    let mut time = 0.;
    let mut event_pump = sdl.event_pump()?;
    let mut camera = Camera::default();
    
    'game: loop {
        let current_time = unsafe { SDL_GetTicksNS() } as f32 / 1e9;
        let delta = current_time - time;
        let look_sensitivity = 100. * delta;
        time = current_time;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'game;
                }
                Event::MouseMotion {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mousestate: _,
                    x: _,
                    y: _,
                    xrel,
                    yrel,
                } => {
                    camera.rotate(xrel * look_sensitivity, yrel * look_sensitivity);
                }
                _ => {}
            }
        }
        keyboard_event_handler(&event_pump, &mut camera, delta);

        let cbuffer = CameraBuffer {
            proj_view: camera.projection(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32) * camera.view(),
            model: Mat4::IDENTITY,
            normal: Mat4::IDENTITY
        };

        let lbuffer = LightingBuffer {
            ambient_color: Vec3::new(0.25, 0.25, 0.5),
            band_count: 5.,
            light_dir: Vec3::new(0.4, 0.8, 0.5),
            _pad0: 0.,
            light_color: Vec3::new(1., 1., 0.8),
        };

        let obuffer = OutlineBuffer {
            color: Vec3::ZERO,
            thickness: 0.05,
        };

        let mut cmdbuffer = device.acquire_command_buffer()?;
        let swapchain_texture = cmdbuffer.wait_and_acquire_swapchain_texture(&window)?;
        let color_target = [ColorTargetInfo::default()
            .with_clear_color(Color::RGB(50, 100, 200))
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_texture(&swapchain_texture)];

        let color_target2 = [ColorTargetInfo::default()
            .with_clear_color(Color::RGB(50, 100, 200))
            .with_load_op(LoadOp::LOAD)
            .with_store_op(StoreOp::STORE)
            .with_texture(&swapchain_texture)];
        let render_pass =
            device.begin_render_pass(&cmdbuffer, &color_target, Some(&depth_info))?;
        pipeline.draw(
            &cmdbuffer,
            &render_pass,
            &cbuffer,
            &lbuffer,
            &obuffer,
            &bindings,
            &index_binding,
            &tex_samp_bind,
            &chunk
        );
        device.end_render_pass(render_pass);
        ui.render(&mut sdl, &device, &window, &event_pump, &mut cmdbuffer, &color_target2, |ui| {
            ui::fps(ui, 1.0/delta);
        });
        let _ = cmdbuffer.submit()?;
    }
    Ok(())
}

fn keyboard_event_handler(event_pump: &sdl3::EventPump, camera: &mut Camera, delta: f32) {
    let keyboard_state = sdl3::keyboard::KeyboardState::new(event_pump);
    let camera_motion = Vec2::new(
        if keyboard_state.is_scancode_pressed(Scancode::W) {
            1.
        } else if keyboard_state.is_scancode_pressed(Scancode::S) {
            -1.
        } else {
            0.
        },
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            1.
        } else if keyboard_state.is_scancode_pressed(Scancode::A) {
            -1.
        } else {
            0.
        },
    )
    .normalize_or_zero() * 10.;
    camera.move_to(
        camera.position
            + (camera.front * camera_motion.x * delta)
            + (camera.front.cross(camera.up) * camera_motion.y * delta),
    );
}

fn create_depth_texture(device: &Device) -> Result<(Texture<'static>, DepthStencilTargetInfo)> {
    let create_info = TextureCreateInfo::default()
        .with_format(TextureFormat::D32Float)
        .with_type(TextureType::_2D)
        .with_usage(TextureUsage::DEPTH_STENCIL_TARGET)
        .with_width(800)
        .with_height(600)
        .with_layer_count_or_depth(1)
        .with_num_levels(1);
    let mut texture = device.create_texture(create_info)?;
    let target_info = DepthStencilTargetInfo::default()
        .with_clear_depth(1.)
        .with_cycle(true)
        .with_load_op(LoadOp::CLEAR)
        .with_store_op(StoreOp::DONT_CARE)
        .with_stencil_load_op(LoadOp::CLEAR)
        .with_stencil_store_op(StoreOp::STORE)
        .with_texture(&mut texture);
    Ok((texture, target_info))
}

// ideally this should get queried from the shaders
struct ShaderDesc {
    entry_point: &'static CStr,
    samplers: u32,
    uniform_buffers: u32,
    storage_textures: u32,
    storage_buffers: u32,
}

const VERTEX_SHADER: ShaderDesc = ShaderDesc {
    entry_point: c"vertex",
    samplers: 0,
    uniform_buffers: 1,
    storage_buffers: 0,
    storage_textures: 0,
};

const FRAG_SHADER: ShaderDesc = ShaderDesc {
    entry_point: c"fragment",
    samplers: 1,
    uniform_buffers: 1,
    storage_buffers: 0,
    storage_textures: 0,
};

const VERT_OUTLINE: ShaderDesc = ShaderDesc {
    entry_point: c"vertex",
    samplers: 0,
    uniform_buffers: 2,
    storage_buffers: 0,
    storage_textures: 0,
};

const FRAG_OUTLINE: ShaderDesc = ShaderDesc {
    entry_point: c"fragment",
    samplers: 0,
    uniform_buffers: 1,
    storage_buffers: 0,
    storage_textures: 0,
};

fn create_shaders(device: &Device, path: String, vert: ShaderDesc, frag: ShaderDesc) -> Result<(Shader, Shader)> {
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

struct Renderer {
    main: GraphicsPipeline,
    outline: GraphicsPipeline,
}

impl Renderer {
    fn new(window: &Window, device: &Device, path: String) -> Result<Self> {
        let (vertex_shader, frag_shader) = create_shaders(device, path.clone() + "main.spv", VERTEX_SHADER, FRAG_SHADER)?;
        let color_target = [ColorTargetDescription::default()
            .with_blend_state(
                ColorTargetBlendState::new()
                    .with_enable_blend(true)
                    .with_color_blend_op(BlendOp::Add)
                    .with_alpha_blend_op(BlendOp::Add)
                    .with_src_color_blendfactor(BlendFactor::SrcAlpha)
                    .with_dst_color_blendfactor(BlendFactor::OneMinusSrcAlpha)
                    .with_src_alpha_blendfactor(BlendFactor::SrcAlpha)
                    .with_dst_alpha_blendfactor(BlendFactor::OneMinusSrcAlpha),
            )
            .with_format(device.get_swapchain_texture_format(window))];
        let target_info = GraphicsPipelineTargetInfo::new()
            .with_color_target_descriptions(&color_target)
            .with_has_depth_stencil_target(true)
            .with_depth_stencil_format(TextureFormat::D32Float);
        let main = device
            .create_graphics_pipeline()
            .with_vertex_shader(&vertex_shader)
            .with_fragment_shader(&frag_shader)
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_vertex_input_state(
                VertexInputState::default()
                    .with_vertex_buffer_descriptions(&[Vertex::buffer_desc()])
                    .with_vertex_attributes(Vertex::attributes().as_slice()),
            )
            .with_target_info(target_info)
            .with_depth_stencil_state(
                DepthStencilState::default()
                    .with_enable_depth_test(true)
                    .with_enable_depth_write(true)
                    .with_compare_op(CompareOp::Less)
                    .with_enable_stencil_test(false),
            )
            .build()?;

        //
        // outline pipeline
        //
        let outline_target_info = GraphicsPipelineTargetInfo::new()
            .with_color_target_descriptions(&color_target)
            .with_has_depth_stencil_target(true)
            .with_depth_stencil_format(TextureFormat::D32Float);

        let (vertex_shader, frag_shader) = create_shaders(device, path + "/outline.spv", VERT_OUTLINE, FRAG_OUTLINE)?;
        let outline = device
            .create_graphics_pipeline()
            .with_vertex_shader(&vertex_shader)
            .with_fragment_shader(&frag_shader)
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_vertex_input_state(
                VertexInputState::default()
                    .with_vertex_buffer_descriptions(&[Vertex::buffer_desc()])
                    .with_vertex_attributes(Vertex::attributes().as_slice()),
            )
            .with_target_info(outline_target_info)
            .with_rasterizer_state(RasterizerState::default().with_cull_mode(CullMode::Front))
            .build()?;

        Ok(Renderer { main, outline })
    }

    fn draw(
        &self,
        cmdbuffer: &CommandBuffer,
        render_pass: &RenderPass,
        camera_buffer: &CameraBuffer,
        light_buffer: &LightingBuffer,
        outline_buffer: &OutlineBuffer,
        bindings: &[BufferBinding],
        index_binding: &BufferBinding,
        tex_samp_bind: &[TextureSamplerBinding<'_>],
        chunk: &Chunk
    ) {
        //
        // outline
        //
        render_pass.bind_graphics_pipeline(&self.outline);
        cmdbuffer.push_vertex_uniform_data(0, camera_buffer);
        cmdbuffer.push_vertex_uniform_data(1, &outline_buffer.thickness);
        cmdbuffer.push_fragment_uniform_data(0, &outline_buffer.color);
        render_pass.bind_vertex_buffers(0, bindings);
        render_pass.bind_index_buffer(index_binding, IndexElementSize::_32BIT);
        render_pass.draw_indexed_primitives(chunk.indices.len() as u32, 1, 0, 0, 0);
        //
        // main
        //
        render_pass.bind_graphics_pipeline(&self.main);
        cmdbuffer.push_vertex_uniform_data(0, camera_buffer);
        cmdbuffer.push_fragment_uniform_data(0, light_buffer);
        render_pass.bind_vertex_buffers(0, bindings);
        render_pass.bind_index_buffer(index_binding, IndexElementSize::_32BIT);
        render_pass.bind_fragment_samplers(0, tex_samp_bind);
        render_pass.draw_indexed_primitives(chunk.indices.len() as u32, 1, 0, 0, 0);

    }
}

fn upload_data<T>(
    device: &Device,
    copy_pass: &CopyPass,
    vertex_buffer: &sdl3::gpu::Buffer,
    data: &[T],
) -> Result<()>
where
    T: Copy,
{
    let size = (size_of::<T>() * data.len()) as u32;
    let transfer_buffer = device
        .create_transfer_buffer()
        .with_size(size)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;
    let mut mem = transfer_buffer.map(device, false);
    mem.mem_mut().copy_from_slice(data);
    mem.unmap();
    let location = TransferBufferLocation::default()
        .with_offset(0)
        .with_transfer_buffer(&transfer_buffer);
    let region = BufferRegion::default()
        .with_buffer(vertex_buffer)
        .with_offset(0)
        .with_size(size);
    copy_pass.upload_to_gpu_buffer(location, region, false);
    Ok(())
}

fn upload_texture(
    device: &Device,
    copy_pass: &CopyPass,
    path: &str,
    texture: &Texture<'static>,
) -> Result<()> {
    let img = image::open(path)?;
    let bytes = img.to_rgba8();
    let transfer_buffer = device
        .create_transfer_buffer()
        .with_size(img.width() * img.height() * 4 * size_of::<u8>() as u32)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;
    let mut memmap = transfer_buffer.map(device, false);
    memmap.mem_mut().copy_from_slice(bytes.as_raw());
    for i in 0..6 {
        copy_pass.upload_to_gpu_texture(
            TextureTransferInfo::new()
                .with_transfer_buffer(&transfer_buffer)
                .with_offset(i * 32 * 32 * 4 * size_of::<u8>() as u32),
            TextureRegion::new()
                .with_texture(texture)
                .with_depth(1)
                .with_height(32)
                .with_width(32)
                .with_layer(i)
                .with_mip_level(0),
            false,
        );
    }
    Ok(())
}

fn create_texture_sampler(device: &Device) -> Result<(Texture<'static>, Sampler)> {
    let sampler_info = SamplerCreateInfo::default();
    let sampler = device.create_sampler(sampler_info)?;
    let texture_info = TextureCreateInfo::default()
        .with_type(TextureType::_2DArray)
        .with_format(TextureFormat::R8g8b8a8Unorm)
        .with_usage(TextureUsage::SAMPLER)
        .with_height(32)
        .with_width(32)
        .with_layer_count_or_depth(6)
        .with_num_levels(1)
        .with_sample_count(SampleCount::NoMultiSampling);
    let texture = device.create_texture(texture_info)?;
    Ok((texture, sampler))
}
