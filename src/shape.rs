use std::mem::swap;

use crate::*;
use util::vec3::*;

pub enum Mesh {
    Sphere { radius: f32 },
    Cube { size: Vec3 },
    InfinitePlane,
    CompositeShape { shapes: Vec<Shape> },
}

pub struct Shape {
    material: Material,
    transform: Transform,
    mesh: Mesh,
}

impl Shape {
    pub fn new(material: Material, transform: Transform, mesh: Mesh) -> Shape {
        Shape {
            material,
            transform,
            mesh,
        }
    }

    pub fn intersect(&self, ray: Ray) -> Result<Intersection, ()> {
        let local_ray = self.transform.inv_transform_ray(ray);
        match self.intersect_local(local_ray) {
            Ok(intersection) => Ok(self.transform.transform_intersection(intersection)),
            Err(()) => Err(()),
        }
    }

    fn intersect_local(&self, ray: Ray) -> Result<Intersection, ()> {
        match &self.mesh {
            Mesh::Sphere { radius } => {
                let a = ray.dir.dot(ray.dir);
                let b = 2.0 * ray.dir.dot(ray.pos);
                let c = ray.pos.dot(ray.pos) - radius.powi(2);

                let (mut t0, mut t1);
                let d: f32 = b.powi(2) - 4.0 * a * c;
                if d < 0.0 {
                    return Err(());
                } else if d == 0.0 {
                    t0 = -0.5 * b / a;
                } else {
                    let q = if b > 0.0 {
                        -0.5 * (b + d.sqrt())
                    } else {
                        -0.5 * (b - d.sqrt())
                    };
                    t0 = q / a;
                    t1 = c / q;
                    if t0 > t1 {
                        swap(&mut t0, &mut t1);
                    }
                }
                if t0 < 0.0 {
                    Err(())
                } else {
                    let intersect_point = ray.pos + t0 * ray.dir;
                    Ok(Intersection {
                        t: t0,
                        pos: intersect_point,
                        normal: intersect_point.normalize(),
                        local_frame: Mat4::identity(),
                        material: self.material,
                    })
                }
            }
            Mesh::Cube { size } => {
                let inv_x = 1.0 / ray.dir.x;
                let inv_y = 1.0 / ray.dir.y;
                let inv_z = 1.0 / ray.dir.z;
                let t1 = (-size.x * 0.5 - ray.pos.x) * inv_x;
                let t2 = (size.x * 0.5 - ray.pos.x) * inv_x;
                let t3 = (-size.y * 0.5 - ray.pos.y) * inv_y;
                let t4 = (size.y * 0.5 - ray.pos.y) * inv_y;
                let t5 = (-size.z * 0.5 - ray.pos.z) * inv_z;
                let t6 = (size.z * 0.5 - ray.pos.z) * inv_z;
                let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
                let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
                if tmax < 0.0 || tmin > tmax {
                    Err(())
                } else {
                    let pos = ray.pos + tmin * ray.dir;
                    let normalized_pos = Vec3::new(
                        if pos.x / size.x * 2.0 < 0.99999 {
                            0.0
                        } else {
                            1.0
                        },
                        if pos.y / size.y * 2.0 < 0.99999 {
                            0.0
                        } else {
                            1.0
                        },
                        if pos.z / size.z * 2.0 < 0.99999 {
                            0.0
                        } else {
                            1.0
                        },
                    );
                    Ok(Intersection {
                        t: tmin,
                        pos: pos,
                        normal: normalized_pos.normalize(),
                        local_frame: Mat4::identity(),
                        material: self.material,
                    })
                }
            }
            Mesh::InfinitePlane => {
                if ray.dir.y == 0.0 {
                    return Err(());
                }
                let intersection_t = ray.pos.y / -ray.dir.y;
                if intersection_t < 0.0 {
                    Err(())
                } else {
                    Ok(Intersection {
                        t: intersection_t,
                        pos: ray.pos + intersection_t * ray.dir,
                        normal: Vec3::new(0.0, if ray.pos.y > 0.0 { 1.0 } else { -1.0 }, 0.0),
                        local_frame: Mat4::identity(),
                        material: self.material,
                    })
                }
            }
            Mesh::CompositeShape { shapes } => {
                shapes
                    .iter()
                    .fold(
                        Err(()),
                        |cur: Result<Intersection, ()>, shape| match shape.intersect(ray) {
                            Ok(info) => match cur {
                                Ok(cur_info) => {
                                    if info.t < cur_info.t {
                                        Ok(info)
                                    } else {
                                        Ok(cur_info)
                                    }
                                }
                                Err(_) => Ok(info),
                            },
                            Err(_) => cur,
                        },
                    )
            }
        }
    }
}
