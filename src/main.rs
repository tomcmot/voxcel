use std::{ffi::CStr, fs};

use anyhow::{Result};
use glam::{Mat4, Vec2, Vec3, camera::rh::{proj::directx, view::look_to_mat4}};
use sdl3::{
    event::Event, gpu::{
        BlendFactor, BlendOp, BufferBinding, BufferRegion, BufferUsageFlags, ColorTargetBlendState, ColorTargetDescription, ColorTargetInfo, CompareOp, DepthStencilState, DepthStencilTargetInfo, Device, GraphicsPipelineTargetInfo, IndexElementSize, LoadOp, PrimitiveType, SampleCount, Sampler, SamplerCreateInfo, ShaderFormat, ShaderStage, StoreOp, Texture, TextureCreateInfo, TextureFormat, TextureRegion, TextureSamplerBinding, TextureTransferInfo, TextureType, TextureUsage, TransferBufferLocation, TransferBufferUsage, VertexAttribute, VertexBufferDescription, VertexElementFormat::{self}, VertexInputState,
    }, keyboard::{Keycode, Scancode}, pixels::Color, sys::timer::SDL_GetTicksNS, video::Window,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: Vec3,
    uv: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy)]struct Camera {
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
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::new(0.,0.,5.),
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
    fn projection(&self) -> Mat4 {
        directx::perspective(self.zoom.to_radians(), 800./600., 0.01, 200.)
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
            if INVERT_Y { -pitch_rad.sin() } else { pitch_rad.sin()} ,
            yaw_rad.sin() * pitch_rad.cos(),
        ).normalize();
    }
}

#[repr(C)]
struct TimeUniform {
    time: f32,
}

const VERTICES: [Vertex; 24] = [                                                                                                                          
       Vertex { position: Vec3::new(-0.5,  0.5, 0.5), uv: Vec3::new(0., 0., 0.) }, // Top Left Front                                                                                                           
       Vertex { position: Vec3::new( 0.5,  0.5, 0.5), uv: Vec3::new(1., 0., 0.) }, // Top Right Front                                                                                                         
       Vertex { position: Vec3::new(-0.5, -0.5, 0.5), uv: Vec3::new(0., 1., 0.) }, // Bottom Left Front                                                                                                        
       Vertex { position: Vec3::new( 0.5, -0.5, 0.5), uv: Vec3::new(1., 1., 0.) }, // Bottom Right Front   
                                            
       Vertex { position: Vec3::new(-0.5,  0.5, -0.5), uv: Vec3::new(1., 0.,1.) }, // Top Left Back                                                                                                            
       Vertex { position: Vec3::new( 0.5,  0.5, -0.5), uv: Vec3::new(0., 0.,1.) }, // Top Right Back                                                                                                          
       Vertex { position: Vec3::new(-0.5, -0.5, -0.5), uv: Vec3::new(1., 1.,1.) }, // Bottom Left Back                                                                                                         
       Vertex { position: Vec3::new( 0.5, -0.5, -0.5), uv: Vec3::new(0., 1.,1.) }, // Bottom Right Back  
                                                                                             
       Vertex { position: Vec3::new(-0.5,  0.5, 0.5), uv: Vec3::new(0., 0., 2.) }, // Top Left Front                                                                                                                 
       Vertex { position: Vec3::new( 0.5,  0.5, 0.5), uv: Vec3::new(1., 0., 2.) }, // Top Right Front
       Vertex { position: Vec3::new(-0.5,  0.5, -0.5), uv: Vec3::new(0., 1.,2.) }, // Top Left Back                                                                                                           
       Vertex { position: Vec3::new( 0.5,  0.5, -0.5), uv: Vec3::new(1., 1.,2.) }, // Top Right Back
 
       Vertex { position: Vec3::new(-0.5, -0.5, 0.5), uv: Vec3::new(0., 0., 3.) }, // Bottom Left Front                                                                                                        
       Vertex { position: Vec3::new( 0.5, -0.5, 0.5), uv: Vec3::new(1., 0., 3.) }, // Bottom Right Front                                                                                                       
       Vertex { position: Vec3::new(-0.5, -0.5, -0.5), uv: Vec3::new(0., 1.,3.) }, // Bottom Left Back                                                                                                         
       Vertex { position: Vec3::new( 0.5, -0.5, -0.5), uv: Vec3::new(1., 1.,3.) }, // Bottom Right Back    

       Vertex { position: Vec3::new(-0.5,  0.5, 0.5), uv: Vec3::new(1., 0., 4.) }, // Top Left Front                                                                                                      
       Vertex { position: Vec3::new(-0.5, -0.5, 0.5), uv: Vec3::new(1., 1., 4.) }, // Bottom Left Front    
       Vertex { position: Vec3::new(-0.5,  0.5, -0.5), uv: Vec3::new(0., 0.,4.) }, // Top Left Back                                                                                                            
       Vertex { position: Vec3::new(-0.5, -0.5, -0.5), uv: Vec3::new(0., 1.,4.) }, // Bottom Left Back      

       Vertex { position: Vec3::new( 0.5,  0.5, 0.5), uv: Vec3::new(0., 0., 5.) }, // Top Right Front                                                                                                      
       Vertex { position: Vec3::new( 0.5, -0.5, 0.5), uv: Vec3::new(0., 1., 5.) }, // Bottom Right Front                                                                                                         
       Vertex { position: Vec3::new( 0.5,  0.5, -0.5), uv: Vec3::new(1., 0.,5.) }, // Top Right Back                                                                                                          
       Vertex { position: Vec3::new( 0.5, -0.5, -0.5), uv: Vec3::new(1., 1.,5.) }, // Bottom Right Back  
];

const INDEXES: [u32; 36] = [
    0,1,2,2,3,1,
    4,5,6,6,7,5,
    8,9,10,10,11,9,
    12,13,14,14,15,13,
    16,17,18,18,19,17,
    20,21,22,22,23,21
];

fn main() -> Result<()> {
    let _ = sdl3::hint::set(sdl3::hint::names::RENDER_VULKAN_DEBUG, "1");
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let mut window = video
        .window("voxcell", 800, 600)
        .position_centered()
        .vulkan()
        .build()?;
    window.set_mouse_grab(true);
    sdl.mouse().set_relative_mouse_mode(&window, true);
    let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;

    let vertex_buffer = device
        .create_buffer()
        .with_size((VERTICES.len() * size_of::<Vertex>()) as u32)
        .with_usage(BufferUsageFlags::VERTEX)
        .build()?;
    let index_buffer = device.create_buffer()
        .with_size((INDEXES.len() * size_of::<u32>()) as u32)
        .with_usage(BufferUsageFlags::INDEX)
        .build()?;
    upload_data(&device, &vertex_buffer, &VERTICES)?;
    upload_data(&device, &index_buffer, &INDEXES)?;
    let (texture, sampler) = create_texture_sampler(&device)?;
    let (_depth_texture, depth_info) = create_depth_texture(&device)?;
    let pipeline = create_pipeline(&window, &device)?;
    let mut time = TimeUniform {
        time: 0.
    };
    let mut event_pump = sdl.event_pump()?;
    let mut camera = Camera::default();
    'game: loop {
        let current_time = unsafe {SDL_GetTicksNS()} as f32 / 1e9;
        let delta = current_time - time.time;
        let look_sensitivity = 1. * delta;
        time.time = current_time;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'game;
                },
                Event::MouseMotion {  timestamp:_, window_id: _, which: _, mousestate:_, x:_, y: _, xrel, yrel } => {
                    camera.rotate(xrel * look_sensitivity, yrel * look_sensitivity);
                },
                _ => {}
            }
        }
        keyboard_event_handler(&event_pump, &mut camera, delta);
        let mut cmdbuffer = device.acquire_command_buffer()?;
        let swapchain_texture = cmdbuffer.wait_and_acquire_swapchain_texture(&window)?;
        let color_target = ColorTargetInfo::default()
            .with_clear_color(Color::RGB(50, 100, 200))
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_texture(&swapchain_texture);

        let render_pass = device.begin_render_pass(&cmdbuffer, &[color_target], Some(&depth_info))?;
        render_pass.bind_graphics_pipeline(&pipeline);
        let cbuffer = CameraBuffer {
            proj_view: camera.projection() * camera.view(),
            model: Mat4::IDENTITY
        };
        cmdbuffer.push_vertex_uniform_data(0, &cbuffer);
        cmdbuffer.push_fragment_uniform_data(0, &time);
        let bindings = [BufferBinding::default()
            .with_buffer(&vertex_buffer)
            .with_offset(0)];
        render_pass.bind_vertex_buffers(0, &bindings);
        render_pass.bind_index_buffer(&BufferBinding::default().with_buffer(&index_buffer).with_offset(0), IndexElementSize::_32BIT);
        let tex_samp_bind = TextureSamplerBinding::default()
            .with_sampler(&sampler)
            .with_texture(&texture);
        render_pass.bind_fragment_samplers(0, &[tex_samp_bind]);
        render_pass.draw_indexed_primitives(INDEXES.len() as u32, 1, 0, 0, 0);
        device.end_render_pass(render_pass);
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
        }).normalize_or_zero();
    camera.move_to(camera.position + (camera.front * camera_motion.x * delta) + (camera.front.cross(camera.up) * camera_motion.y * delta));
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

struct ShaderDesc {
    entry_point: &'static CStr,
    samplers: u32,
    uniform_buffers: u32,
    storage_textures: u32,
    storage_buffers: u32
}

const VERTEX_SHADER: ShaderDesc = ShaderDesc {
    entry_point: c"vertMain",
    samplers: 0,
    uniform_buffers: 1,
    storage_buffers: 0,
    storage_textures: 0
};

const FRAG_SHADER: ShaderDesc = ShaderDesc {
    entry_point: c"fragMain",
    samplers: 1,
    uniform_buffers: 1,
    storage_buffers: 0,
    storage_textures: 0,
};

fn create_pipeline(window: &Window, device: &Device) -> Result<sdl3::gpu::GraphicsPipeline, anyhow::Error> {
    // todo feed path in
    let code = fs::read("slang.spv")?;
    let vertex_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, code.as_slice(), ShaderStage::Vertex)
        .with_entrypoint(VERTEX_SHADER.entry_point)
        .with_samplers(VERTEX_SHADER.samplers)
        .with_uniform_buffers(VERTEX_SHADER.uniform_buffers)
        .with_storage_textures(VERTEX_SHADER.storage_textures)
        .with_storage_buffers(VERTEX_SHADER.storage_buffers)
        .build()?;
    let frag_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, code.as_slice(), ShaderStage::Fragment)
        .with_entrypoint(FRAG_SHADER.entry_point)
        .with_samplers(FRAG_SHADER.samplers)
        .with_uniform_buffers(FRAG_SHADER.uniform_buffers)
        .with_storage_textures(FRAG_SHADER.storage_textures)
        .with_storage_buffers(FRAG_SHADER.storage_buffers)
        .build()?;
    let vertex_buffer_desc = VertexBufferDescription::default()
        .with_slot(0)
        .with_input_rate(sdl3::gpu::VertexInputRate::Vertex)
        .with_instance_step_rate(0)
        .with_pitch(size_of::<Vertex>() as u32);
    let vertex_attrib = [
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
    ];
    let color_target = ColorTargetDescription::default()
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
        .with_format(device.get_swapchain_texture_format(window));
    Ok(device
        .create_graphics_pipeline()
        .with_vertex_shader(&vertex_shader)
        .with_fragment_shader(&frag_shader)
        .with_primitive_type(PrimitiveType::TriangleList)
        .with_vertex_input_state(
            VertexInputState::default()
                .with_vertex_buffer_descriptions(&[vertex_buffer_desc])
                .with_vertex_attributes(&vertex_attrib),
        )
        .with_target_info(
            GraphicsPipelineTargetInfo::new()
            .with_color_target_descriptions(&[color_target])
            .with_has_depth_stencil_target(true)
            .with_depth_stencil_format(TextureFormat::D32Float),
        )
        .with_depth_stencil_state(
            DepthStencilState::default()
            .with_enable_depth_test(true)
            .with_enable_depth_write(true)
            .with_compare_op(CompareOp::Less)
            .with_enable_stencil_test(false)
        )
        .build()?)
}

fn upload_data<T>(device: &Device, vertex_buffer: &sdl3::gpu::Buffer, data: &[T]) -> Result<()> 
    where T: Copy
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
    let init_cmd_buffer = device.acquire_command_buffer()?;
    let copy_pass = device.begin_copy_pass(&init_cmd_buffer)?;
    let location = TransferBufferLocation::default()
        .with_offset(0)
        .with_transfer_buffer(&transfer_buffer);
    let region = BufferRegion::default()
        .with_buffer(vertex_buffer)
        .with_offset(0)
        .with_size(size);
    copy_pass.upload_to_gpu_buffer(location, region, false);
    device.end_copy_pass(copy_pass);
    let _ = init_cmd_buffer.submit()?;
    Ok(())
}

fn upload_texture(device: &Device) -> Result<Texture<'static>> {
    // todo feed path in
    let i = image::open("assets/debug.png")?;
    let bytes = i.to_rgba8();
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
    let command_buffer = device.acquire_command_buffer()?;
    let copy_pass= device.begin_copy_pass(&command_buffer)?;
    let transfer_buffer = device.create_transfer_buffer()
        .with_size(i.width() * i.height() * 4 *size_of::<u8>() as u32)
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
                .with_texture(&texture)
                .with_depth(1)
                .with_height(32)
                .with_width(32)
                .with_layer(i)
                .with_mip_level(0)
        , false);
    }
    device.end_copy_pass(copy_pass);
    let _ = command_buffer.submit()?;
    Ok(texture)
}

fn create_texture_sampler(device: &Device) -> Result<(Texture<'static>, Sampler)> {
    let sampler_info = SamplerCreateInfo::default();
    let sampler = device.create_sampler(sampler_info)?;
    let texture = upload_texture(device)?;
    Ok((texture, sampler))
}
