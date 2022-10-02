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
                obj.faces
                    .iter()
                    .fold(None, |cur: Option<Intersection>, face| {
                        let p0 = obj.points[face[0].0];
                        let p1 = obj.points[face[1].0];
                        let p2 = obj.points[face[2].0];
                        let v1 = p1 - p0;
                        let v2 = p2 - p0;
                        let n = v1.cross(v2).normalize();

                        // 평면의 방정식
                        // Ax + By + Cz + D = 0, D = - Ax0 - By0 - Cz0
                        // A, B, C = n
                        // x0, y0, z0 = p0

                        if ray.dir.dot(n) == 0.0 {
                            return cur;
                        }
                        let dist = (n.x * ray.pos.x + n.y * ray.pos.y + n.z * ray.pos.z
                            - n.x * p0.x
                            - n.y * p0.y
                            - n.z * p0.z)
                            .abs();

                        let intersection_t = dist / -ray.dir.dot(n);
                        if intersection_t < 0.0 {
                            cur
                        } else {
                            let pos = ray.pos + intersection_t * ray.dir;
                            let pos_p0 = pos - p0;
                            let a1 = v1.angle(v2);
                            let a2 = pos_p0.angle(v1);
                            let a3 = pos_p0.angle(v2);
                            if a2 >= a1 || a3 >= a1 {
                                return cur;
                            }

                            let v1 = p0 - p1;
                            let v2 = p2 - p1;
                            let pos_p0 = pos - p1;
                            let a1 = v1.angle(v2);
                            let a2 = pos_p0.angle(v1);
                            let a3 = pos_p0.angle(v2);
                            if a2 >= a1 || a3 >= a1 {
                                return cur;
                            }

                            let v1 = p0 - p2;
                            let v2 = p1 - p2;
                            let pos_p0 = pos - p2;
                            let a1 = v1.angle(v2);
                            let a2 = pos_p0.angle(v1);
                            let a3 = pos_p0.angle(v2);
                            if a2 >= a1 || a3 >= a1 {
                                return cur;
                            }

                            let n0 = obj.normals[face[0].1];
                            let n1 = obj.normals[face[1].1];
                            let n2 = obj.normals[face[2].1];
                            let d0 = 1.0 / (pos - p0).length();
                            let d1 = 1.0 / (pos - p1).length();
                            let d2 = 1.0 / (pos - p2).length();
                            // let n = (d0 * n0 + d1 * n1 + d2 * n2).normalize();

                            let info = Intersection {
                                t: intersection_t,
                                pos,
                                normal: n,
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
