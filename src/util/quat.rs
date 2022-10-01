use super::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Quat {
        Quat { x, y, z, w }
    }

    pub fn default() -> Quat {
        Quat::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quat {
        let v = axis * (angle / 2.0).sin();
        Quat::new(v.x, v.y, v.z, (angle / 2.0).cos())
    }
}
