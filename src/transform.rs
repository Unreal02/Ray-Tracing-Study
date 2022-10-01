use crate::{
    util::{mat4::Mat4, vec4::Vec4},
    *,
};

#[derive(Clone)]
pub struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Transform {
    pub fn default() -> Transform {
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_t(t: Vec3) -> Transform {
        Transform {
            translation: t,
            rotation: Quat::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_tr(t: Vec3, r: Quat) -> Transform {
        Transform {
            translation: t,
            rotation: r,
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_trs(t: Vec3, r: Quat, s: Vec3) -> Transform {
        Transform {
            translation: t,
            rotation: r,
            scale: s,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        let matrix_t = Mat4::from_translation(self.translation);
        let matrix_r = Mat4::from_quat(self.rotation);
        let matrix_s = Mat4::from_scale(self.scale);

        matrix_t * matrix_r * matrix_s
    }

    pub fn inv_transform_ray(&self, ray: Ray) -> Ray {
        let m = self.matrix();
        let m_inv = m.invert().unwrap();
        Ray {
            pos: Vec3::from_vec4(m_inv * Vec4::from_vec3(ray.pos, 1.0)),
            dir: Vec3::from_vec4(m_inv * Vec4::from_vec3(ray.dir, 0.0)),
        }
    }

    pub fn transform_intersection(&self, local_intersection: Intersection) -> Intersection {
        let m = self.matrix();
        Intersection {
            t: local_intersection.t,
            pos: Vec3::from_vec4(m * Vec4::from_vec3(local_intersection.pos, 1.0)),
            normal: Vec3::from_vec4(
                m.invert().unwrap().transpose() * Vec4::from_vec3(local_intersection.normal, 0.0),
            ),
            local_frame: m * local_intersection.local_frame,
            material: local_intersection.material,
        }
    }
}
