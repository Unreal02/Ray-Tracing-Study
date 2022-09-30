use std::mem::swap;

use cgmath::{num_traits::Pow, InnerSpace, Matrix4, SquareMatrix, Vector3};

use crate::*;

pub enum Mesh {
    Sphere { radius: f32 },
    Cube { size: Vector3<f32> },
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
                let c = ray.pos.dot(ray.pos) - radius.pow(2);

                let (mut t0, mut t1);
                let d: f32 = b.pow(2) - 4.0 * a * c;
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
                        local_frame: Matrix4::identity(),
                        material: self.material,
                    })
                }
            }
            Mesh::Cube { size } => {
                let t1 = (-size[0] * 0.5 - ray.pos[0]) / ray.dir[0];
                let t2 = (size[0] * 0.5 - ray.pos[0]) / ray.dir[0];
                let t3 = (-size[1] * 0.5 - ray.pos[1]) / ray.dir[1];
                let t4 = (size[1] * 0.5 - ray.pos[1]) / ray.dir[1];
                let t5 = (-size[2] * 0.5 - ray.pos[2]) / ray.dir[2];
                let t6 = (size[2] * 0.5 - ray.pos[2]) / ray.dir[2];
                let tmin = vec![t1.min(t2), t3.min(t4), t5.min(t6)]
                    .iter()
                    .cloned()
                    .fold(-1.0 / 0.0 /* -inf */, f32::max);
                let tmax = vec![t1.max(t2), t3.max(t4), t5.max(t6)]
                    .iter()
                    .cloned()
                    .fold(1.0 / 0.0 /* inf */, f32::min);
                if tmax < 0.0 || tmin > tmax {
                    Err(())
                } else {
                    let pos = ray.pos + tmin * ray.dir;
                    let normalized_pos = Vector3::new(
                        pos.x / size.x * 2.0,
                        pos.y / size.y * 2.0,
                        pos.z / size.z * 2.0,
                    )
                    .map(|i| if i < 0.99999 { 0.0 } else { 1.0 });
                    Ok(Intersection {
                        t: tmin,
                        pos: pos,
                        normal: normalized_pos.normalize(),
                        local_frame: Matrix4::identity(),
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
                        normal: Vector3::new(0.0, if ray.pos.y > 0.0 { 1.0 } else { -1.0 }, 0.0),
                        local_frame: Matrix4::identity(),
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
