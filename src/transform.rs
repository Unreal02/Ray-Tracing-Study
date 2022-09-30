use crate::*;

#[derive(Clone)]
pub struct Transform {
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}

impl Transform {
    pub fn default() -> Transform {
        Transform {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion {
                v: Vector3::new(0.0, 0.0, 0.0),
                s: 1.0,
            },
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_t(t: Vector3<f32>) -> Transform {
        Transform {
            translation: t,
            rotation: Quaternion {
                v: Vector3::new(0.0, 0.0, 0.0),
                s: 1.0,
            },
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_tr(t: Vector3<f32>, r: Quaternion<f32>) -> Transform {
        Transform {
            translation: t,
            rotation: r,
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_trs(t: Vector3<f32>, r: Quaternion<f32>, s: Vector3<f32>) -> Transform {
        Transform {
            translation: t,
            rotation: r,
            scale: s,
        }
    }

    fn matrix(&self) -> Matrix4<f32> {
        let mut matrix_t = Matrix4::identity();
        matrix_t.w.x = self.translation.x;
        matrix_t.w.y = self.translation.y;
        matrix_t.w.z = self.translation.z;
        let matrix_r = Matrix4::from(self.rotation);
        let mut matrix_s = Matrix4::identity();
        matrix_s.x.x = self.scale.x;
        matrix_s.y.y = self.scale.y;
        matrix_s.z.z = self.scale.z;

        matrix_t * matrix_r * matrix_s
    }

    pub fn inv_transform_ray(&self, ray: Ray) -> Ray {
        let m = self.matrix();
        let m_inv = m.invert().unwrap();
        Ray {
            pos: v4_to_v3(m_inv * v3_to_v4(ray.pos, 1.0)),
            dir: v4_to_v3(m_inv * v3_to_v4(ray.dir, 0.0)),
        }
    }

    pub fn transform_intersection(&self, local_intersection: Intersection) -> Intersection {
        let m = self.matrix();
        Intersection {
            t: local_intersection.t,
            pos: v4_to_v3(m * v3_to_v4(local_intersection.pos, 1.0)),
            normal: v4_to_v3(
                m.invert().unwrap().transpose() * v3_to_v4(local_intersection.normal, 0.0),
            ),
            local_frame: m * local_intersection.local_frame,
            material: local_intersection.material,
        }
    }
}
