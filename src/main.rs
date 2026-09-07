use anyhow::Result;
use glam::{ Vec2,
};
use sdl3::{
    event::Event,
    gpu::{
        ColorTargetInfo, DepthStencilTargetInfo, Device, LoadOp, ShaderFormat, 
        StoreOp, Texture, TextureCreateInfo, TextureFormat, TextureType, TextureUsage,
    },
    keyboard::{Keycode, Scancode},
    pixels::Color,
    sys::timer::SDL_GetTicksNS,
};

mod aabb;
mod app;
mod camera;
mod chunk;
mod chunk_render;
mod ui;
mod world;
mod gpu_mem;
mod shaders;
mod toroid;

use app::App;

use crate::camera::Camera;

// todo these constants should be swapped to be queried at runtime
const SHADER_PATH: &'static str = "assets/";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
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
    device.set_swapchain_parameters(
        &window,
        sdl3::gpu::PresentMode::Immediate,
        sdl3::gpu::SwapchainComposition::Sdr,
    )?;
    let mut ui = ui::UI::new(&device, &window);

    let (_depth_texture, depth_info) = create_depth_texture(&device)?;
    let mut app = App::new(&window, &device, SHADER_PATH.into())?;
    let mut event_pump = sdl.event_pump()?;
    let mut time = unsafe { SDL_GetTicksNS() } as f32 / 1e9;
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
                    app.camera.rotate(xrel * look_sensitivity, yrel * look_sensitivity);
                }
                _ => {}
            }
        }
        keyboard_event_handler(&event_pump, &mut app.camera, delta);
        let mut cmdbuffer = device.acquire_command_buffer()?;
        {
            let copy_pass = device.begin_copy_pass(&cmdbuffer)?;
            app.generate_world(&device, &copy_pass)?;
            device.end_copy_pass(copy_pass);
        }
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
        let render_pass = device.begin_render_pass(&cmdbuffer, &color_target, Some(&depth_info))?;
        app.render(&cmdbuffer, &render_pass);
        device.end_render_pass(render_pass);
        ui.render(
            &mut sdl,
            &device,
            &window,
            &event_pump,
            &mut cmdbuffer,
            &color_target2,
            |ui| {
                ui::fps(ui, 1.0 / delta);
            },
        );
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
    .normalize_or_zero()
        * 10.;
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



