use std::f64::consts::TAU;

use noise::{NoiseFn, Simplex};

pub struct ToroidNoise {
    noise: Simplex,
    size: f64,
}

impl ToroidNoise {
    pub fn new(seed: u32, size: f64) -> Self {
        ToroidNoise { noise: Simplex::new(seed), size }
    }
}

impl NoiseFn<f64,2> for ToroidNoise {
    // accepts x, z
    fn get(&self, point: [f64;2]) -> f64 {
        let ax = point[0] / self.size * TAU;
        let az = point[1] / self.size * TAU;
        self.noise.get([ax.cos(), ax.sin(), az.cos(), az.sin()])
    }
}

impl NoiseFn<f64,3> for ToroidNoise {
    // accepts x, y, z
    fn get(&self, point: [f64;3]) -> f64 {
        let ax = point[0] / self.size * TAU;
        let az = point[2] / self.size * TAU;
        let y = point[1];
        self.noise.get([ax.cos() + y, ax.sin() + y, az.cos() + y, az.sin() + y])
    }
}