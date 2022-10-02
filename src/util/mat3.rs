use crate::*;

pub struct Mat3 {
    x: Vec3,
    y: Vec3,
    z: Vec3,
}

impl Mat3 {
    pub fn from_cols(x: Vec3, y: Vec3, z: Vec3) -> Mat3 {
        Mat3 { x, y, z }
    }

    pub fn determinant(&self) -> f32 {
        self.x.x * (self.y.y * self.z.z - self.z.y * self.y.z)
            - self.y.x * (self.x.y * self.z.z - self.z.y * self.x.z)
            + self.z.x * (self.x.y * self.y.z - self.y.y * self.x.z)
    }
}
