use imgui::Condition::Once;
use imgui_sdl3::ImGuiSdl3;
use sdl3::{gpu::Device, video::Window};
pub struct UI {
    imgui: ImGuiSdl3,
}

impl UI {
    pub fn new(device: &Device, window: &Window) -> Self {
        UI {
            imgui: ImGuiSdl3::new(device, window, |ctx| {
                ctx.set_ini_filename(None);
                ctx.set_log_filename(None);
                ctx.fonts()
                    .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
            }),
        }
    }

    pub fn render<T>(
        &mut self,
        sdl: &mut sdl3::Sdl,
        device: &Device,
        window: &Window,
        event_pump: &sdl3::EventPump,
        command_buffer: &mut sdl3::gpu::CommandBuffer,
        color_targets: &[sdl3::gpu::ColorTargetInfo],
        draw: T,
    ) where
        T: FnMut(&mut imgui::Ui),
    {
        self.imgui.render(
            sdl,
            device,
            window,
            event_pump,
            command_buffer,
            color_targets,
            draw,
        )
    }
}

pub fn fps(ui: &mut imgui::Ui, rate: f32) {
    let flags = imgui::WindowFlags::NO_TITLE_BAR
        | imgui::WindowFlags::NO_RESIZE
        | imgui::WindowFlags::NO_MOVE
        | imgui::WindowFlags::NO_BACKGROUND
        | imgui::WindowFlags::NO_INPUTS; 
    ui.window("Debug")
    .position([10.,10.], Once)
    .size([250., 150.], Once)
    .flags(flags)
    .build(|| {
        ui.text(format!("FPS: {:.1}", rate));
    });
}