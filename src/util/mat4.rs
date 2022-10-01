use std::ops::{Index, IndexMut, Mul};

use super::{mat3::Mat3, quat::Quat, vec3::Vec3, vec4::Vec4};

#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    x: Vec4,
    y: Vec4,
    z: Vec4,
    w: Vec4,
}

impl Mat4 {
    pub fn new(x: Vec4, y: Vec4, z: Vec4, w: Vec4) -> Mat4 {
        Mat4 { x, y, z, w }
    }

    pub fn zero() -> Mat4 {
        Mat4::new(Vec4::zero(), Vec4::zero(), Vec4::zero(), Vec4::zero())
    }

    pub fn identity() -> Mat4 {
        Mat4::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn from_quat(q: Quat) -> Mat4 {
        let x = q.x;
        let y = q.y;
        let z = q.z;
        let w = q.w;
        Mat4::new(
            Vec4::new(
                1.0 - 2.0 * y.powi(2) - 2.0 * z.powi(2),
                2.0 * x * y + 2.0 * z * w,
                2.0 * x * z - 2.0 * y * w,
                0.0,
            ),
            Vec4::new(
                2.0 * x * y - 2.0 * z * w,
                1.0 - 2.0 * x.powi(2) - 2.0 * z.powi(2),
                2.0 * y * z + 2.0 * x * w,
                0.0,
            ),
            Vec4::new(
                2.0 * x * z + 2.0 * y * w,
                2.0 * y * z - 2.0 * x * w,
                1.0 - 2.0 * x.powi(2) - 2.0 * y.powi(2),
                0.0,
            ),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn from_translation(v: Vec3) -> Mat4 {
        Mat4::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(v.x, v.y, v.z, 1.0),
        )
    }

    pub fn from_scale(v: Vec3) -> Mat4 {
        Mat4::new(
            Vec4::new(v.x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, v.y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, v.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn transpose(&self) -> Mat4 {
        let mut m = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                m[i][j] += self[j][i];
            }
        }
        m
    }

    pub fn determinant(&self) -> f32 {
        let v0 = self.x.truncate(0);
        let v1 = self.y.truncate(0);
        let v2 = self.z.truncate(0);
        let v3 = self.w.truncate(0);
        let m0 = Mat3::from_cols(v1, v2, v3);
        let m1 = Mat3::from_cols(v0, v2, v3);
        let m2 = Mat3::from_cols(v0, v1, v3);
        let m3 = Mat3::from_cols(v0, v1, v2);
        self[0][0] * m0.determinant() - self[1][0] * m1.determinant()
            + self[2][0] * m2.determinant()
            - self[3][0] * m3.determinant()
    }

    pub fn invert(&self) -> Option<Mat4> {
        let det = self.determinant();
        if det == 0.0 {
            None
        } else {
            let inv_det = 1.0 / det;
            let t = self.transpose();
            let cf = |i, j| {
                let mat = match i {
                    0 => Mat3::from_cols(t.y.truncate(j), t.z.truncate(j), t.w.truncate(j)),
                    1 => Mat3::from_cols(t.x.truncate(j), t.z.truncate(j), t.w.truncate(j)),
                    2 => Mat3::from_cols(t.x.truncate(j), t.y.truncate(j), t.w.truncate(j)),
                    3 => Mat3::from_cols(t.x.truncate(j), t.y.truncate(j), t.z.truncate(j)),
                    _ => panic!("out of range"),
                };
                let sign = if (i + j) % 2 == 1 { -1.0 } else { 1.0 };
                mat.determinant() * sign * inv_det
            };

            let mut mat = Mat4::zero();
            for i in 0..4 {
                for j in 0..4 {
                    mat[i][j] = cf(i, j);
                }
            }
            Some(mat)
        }
    }
}

impl Index<usize> for Mat4 {
    type Output = Vec4;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("out of range"),
        }
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("out of range"),
        }
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        let mut m = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                m[j][i] = self[0][i] * rhs[j][0]
                    + self[1][i] * rhs[j][1]
                    + self[2][i] * rhs[j][2]
                    + self[3][i] * rhs[j][3];
            }
        }
        m
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::new(
            self[0][0] * rhs[0] + self[1][0] * rhs[1] + self[2][0] * rhs[2] + self[3][0] * rhs[3],
            self[0][1] * rhs[0] + self[1][1] * rhs[1] + self[2][1] * rhs[2] + self[3][1] * rhs[3],
            self[0][2] * rhs[0] + self[1][2] * rhs[1] + self[2][2] * rhs[2] + self[3][2] * rhs[3],
            self[0][3] * rhs[0] + self[1][3] * rhs[1] + self[2][3] * rhs[2] + self[3][3] * rhs[3],
        )
    }
}
