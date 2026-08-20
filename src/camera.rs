
use glam::{
    Mat4, Vec3,
    camera::rh::{proj::directx, view::look_to_mat4},
};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub zoom: f32,
}


impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::new(16., 12., 16.),
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
    pub fn move_to(&mut self, p: Vec3) {
        self.position = p;
    }

    pub fn view(&self) -> Mat4 {
        look_to_mat4(self.position, self.front, self.up)
    }

    /// SDL3 GPU uses the directx Z convention
    pub fn projection(&self, width: f32, height: f32) -> Mat4 {
        directx::perspective(self.zoom.to_radians(), width / height, 0.01, 200.)
    }

    pub fn rotate(&mut self, x_offset: f32, y_offset: f32) {
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