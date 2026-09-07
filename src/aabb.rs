use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

pub struct Plane {
    normal: Vec3,
    distance: f32
}

impl Plane {
    pub fn frustrum_planes(proj_view: Mat4) -> Vec<Self> {
        let r0 = proj_view.row(0);
        let r1 = proj_view.row(1);
        let r2 = proj_view.row(2);
        let r3 = proj_view.row(3);
        let left = to_plane(r3 + r0).normalize();
        let right = to_plane(r3 - r0).normalize();
        let bottom = to_plane(r3 + r1).normalize();
        let top = to_plane(r3 - r1).normalize();
        let near = to_plane(r3 + r2).normalize();
        let far = to_plane(r3 - r2).normalize();
        vec![left, right, bottom, top, near, far]
    }
    
    fn normalize(&self) -> Self {
        let length = self.normal.length();
        Plane {
            normal: self.normal / length,
            distance: self.distance / length
        }
    }

    fn distance(&self, p: Vec3) -> f32 {
        self.normal.dot(p) + self.distance
    }
}

fn to_plane(v: Vec4) -> Plane {
    Plane {
        normal: v.xyz(),
        distance: v.w
    }
}

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3
}

impl AABB {
    pub fn intersects_frustrum(&self, planes: &Vec<Plane>) -> bool {
        planes.iter().all(|p| {
            let x = if p.normal.x >= 0. {
                self.max.x
            } else {
                self.min.x
            };

            let y = if p.normal.y >= 0. {
                self.max.y
            } else {
                self.min.y
            };

            let z = if p.normal.z >= 0. {
                self.max.z
            } else {
                self.min.z
            };
            let positive_vertex = Vec3::new(
                x, y, z
            );
            p.distance(positive_vertex) >= 0.
        })
    }
}