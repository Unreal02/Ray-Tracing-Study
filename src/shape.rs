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
                        let polygon_normal = v1.cross(v2).normalize();

                        // 평면의 방정식
                        // Ax + By + Cz + D = 0, D = - Ax0 - By0 - Cz0
                        // A, B, C = n
                        // x0, y0, z0 = p0

                        if ray.dir.dot(polygon_normal) == 0.0 {
                            return cur;
                        }
                        let dist = polygon_normal.x * ray.pos.x
                            + polygon_normal.y * ray.pos.y
                            + polygon_normal.z * ray.pos.z
                            - polygon_normal.x * p0.x
                            - polygon_normal.y * p0.y
                            - polygon_normal.z * p0.z;

                        let intersection_t = dist / -ray.dir.dot(polygon_normal);
                        if intersection_t < 0.0 {
                            cur
                        } else {
                            let pos = ray.pos + intersection_t * ray.dir;

                            // pos가 polygon 내부에 있는지 판별
                            let v01 = p1 - p0;
                            let v02 = p2 - p0;
                            let v0p = pos - p0;
                            if v01.cross(v0p).dot(v0p.cross(v02)) < 0.0 {
                                return cur;
                            }
                            let v12 = p2 - p1;
                            let v10 = p0 - p1;
                            let v1p = pos - p1;
                            if v12.cross(v1p).dot(v1p.cross(v10)) < 0.0 {
                                return cur;
                            }
                            let v20 = p0 - p2;
                            let v21 = p1 - p2;
                            let v2p = pos - p2;
                            if v20.cross(v2p).dot(v2p.cross(v21)) < 0.0 {
                                return cur;
                            }

                            let n0 = obj.normals[face[0].1];
                            let n1 = obj.normals[face[1].1];
                            let n2 = obj.normals[face[2].1];

                            // p1과 p2에 대해 먼저 interpolate하고 그 점과 p0에 대해 interpolate하기
                            let n12 = polygon_normal.cross(v12).normalize();
                            let dist12 = (n12.x * p0.x + n12.y * p0.y + n12.z * p0.z
                                - n12.x * p1.x
                                - n12.y * p1.y
                                - n12.z * p1.z)
                                .abs();
                            let t012 = dist12 / v0p.dot(n12).abs();
                            let pos12 = p0 + t012 * v0p;
                            let dist1 = (pos12 - p1).length();
                            let dist2 = (pos12 - p2).length();
                            let n12 = (n1 * dist2 + n2 * dist1).normalize();
                            let dist12 = (pos - pos12).length();
                            let dist0 = (pos - p0).length();
                            let n = (n0 * dist12 + n12 * dist0).normalize();

                            let info = Intersection {
                                t: intersection_t,
                                pos,
                                normal: if dist >= 0.0 { n } else { -1.0 * n },
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
