use std::mem::swap;

use crate::*;

pub enum Mesh {
    Sphere { radius: f32 },
    Cube { size: Vec3 },
    InfinitePlane,
    Polygons { obj: Object },
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

    pub fn intersect(&self, ray: Ray) -> Option<Intersection> {
        let local_ray = self.transform.inv_transform_ray(ray);
        match self.intersect_local(local_ray) {
            Some(intersection) => Some(self.transform.transform_intersection(intersection)),
            None => None,
        }
    }

    fn intersect_local(&self, ray: Ray) -> Option<Intersection> {
        match &self.mesh {
            Mesh::Sphere { radius } => {
                let a = ray.dir.dot(ray.dir);
                let b = 2.0 * ray.dir.dot(ray.pos);
                let c = ray.pos.dot(ray.pos) - radius.powi(2);

                let (mut t0, mut t1);
                let d: f32 = b.powi(2) - 4.0 * a * c;
                if d < 0.0 {
                    return None;
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
                    None
                } else {
                    let intersect_point = ray.pos + t0 * ray.dir;
                    Some(Intersection {
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
                    None
                } else {
                    let pos = ray.pos + tmin * ray.dir;
                    let normalized_pos = Vec3::new(
                        if (pos.x / size.x).abs() * 2.0 < 0.99999 {
                            0.0
                        } else {
                            pos.x
                        },
                        if (pos.y / size.y).abs() * 2.0 < 0.99999 {
                            0.0
                        } else {
                            pos.y
                        },
                        if (pos.z / size.z).abs() * 2.0 < 0.99999 {
                            0.0
                        } else {
                            pos.z
                        },
                    )
                    .normalize();
                    Some(Intersection {
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
                    return None;
                }
                let intersection_t = ray.pos.y / -ray.dir.y;
                if intersection_t < 0.0 {
                    None
                } else {
                    Some(Intersection {
                        t: intersection_t,
                        pos: ray.pos + intersection_t * ray.dir,
                        normal: Vec3::new(0.0, if ray.pos.y > 0.0 { 1.0 } else { -1.0 }, 0.0),
                        local_frame: Mat4::identity(),
                        material: self.material,
                    })
                }
            }
            Mesh::Polygons { obj } => {
                obj.polygons
                    .iter()
                    .fold(None, |cur: Option<Intersection>, polygon| {
                        let p0 = polygon.points[0];
                        let p1 = polygon.points[1];
                        let p2 = polygon.points[2];
                        let n0 = polygon.normals[0];
                        let n1 = polygon.normals[1];
                        let n2 = polygon.normals[2];

                        let e1 = p1 - p0;
                        let e2 = p2 - p0;
                        let s = ray.pos - p0;
                        let p = ray.dir.cross(e2);
                        let q = s.cross(e1);

                        let tvw = 1.0 / p.dot(e1) * Vec3::new(q.dot(e2), p.dot(s), q.dot(ray.dir));
                        let intersection_t = tvw.x;
                        let w1 = tvw.y;
                        let w2 = tvw.z;
                        let w0 = 1.0 - w1 - w2;

                        if intersection_t < 0.0
                            || w0 < -0.0
                            || w0 > 1.0
                            || w1 < -0.0
                            || w1 > 1.0
                            || w2 < -0.0
                            || w2 > 1.0
                        {
                            return cur;
                        }

                        let n = (w0 * n0 + w1 * n1 + w2 * n2).normalize();

                        let info = Intersection {
                            t: intersection_t,
                            pos: ray.pos + intersection_t * ray.dir,
                            normal: if ray.dir.dot(e1.cross(e2)) <= 0.0 {
                                n
                            } else {
                                -1.0 * n
                            },
                            local_frame: Mat4::identity(),
                            material: self.material,
                        };
                        match cur {
                            Some(cur_info) => {
                                if info.t < cur_info.t {
                                    Some(info)
                                } else {
                                    Some(cur_info)
                                }
                            }
                            None => Some(info),
                        }
                    })
            }
            Mesh::CompositeShape { shapes } => {
                shapes
                    .iter()
                    .fold(None, |cur: Option<Intersection>, shape| {
                        match shape.intersect(ray) {
                            Some(info) => match cur {
                                Some(cur_info) => {
                                    if info.t < cur_info.t {
                                        Some(info)
                                    } else {
                                        Some(cur_info)
                                    }
                                }
                                None => Some(info),
                            },
                            None => cur,
                        }
                    })
            }
        }
    }
}
