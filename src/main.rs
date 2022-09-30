use cgmath::num_traits::Pow;
use cgmath::{InnerSpace, Matrix, Matrix4, Quaternion, SquareMatrix, Vector3, Vector4};
use image::{ImageBuffer, Rgb, RgbImage};
use std::f32::consts::PI;
use std::mem::swap;
use std::time::Instant;

fn v4_to_v3(v4: Vector4<f32>) -> Vector3<f32> {
    Vector3::new(v4.x, v4.y, v4.z)
}

fn v3_to_v4(v3: Vector3<f32>, w: f32) -> Vector4<f32> {
    Vector4::new(v3.x, v3.y, v3.z, w)
}

#[derive(Clone, Copy)]
struct Material {
    color: Rgb<u8>,
}

#[derive(Clone, Copy)]
struct Ray {
    pos: Vector3<f32>,
    dir: Vector3<f32>,
}

#[derive(Clone)]
struct Intersection {
    t: f32,
    pos: Vector3<f32>,
    normal: Vector3<f32>,
    local_frame: Matrix4<f32>,
    material: Material,
}

#[derive(Clone)]
struct Transform {
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}

impl Transform {
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

    fn inv_transform_ray(&self, ray: Ray) -> Ray {
        let m = self.matrix();
        let m_inv = m.invert().unwrap();
        Ray {
            pos: v4_to_v3(m_inv * v3_to_v4(ray.pos, 1.0)),
            dir: v4_to_v3(m_inv * v3_to_v4(ray.dir, 0.0)),
        }
    }

    fn transform_intersection(&self, local_intersection: Intersection) -> Intersection {
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

fn axis_angle_to_quat(axis: Vector3<f32>, angle: f32) -> Quaternion<f32> {
    Quaternion {
        v: axis * (angle / 2.0).sin(),
        s: (angle / 2.0).cos(),
    }
}

enum Mesh {
    Sphere { radius: f32 },
    Cube { size: Vector3<f32> },
    InfinitePlane,
    CompositeShape { shapes: Vec<Shape> },
}

struct Shape {
    material: Material,
    transform: Transform,
    mesh: Mesh,
}

impl Shape {
    fn new(material: Material, transform: Transform, mesh: Mesh) -> Shape {
        Shape {
            material,
            transform,
            mesh,
        }
    }

    fn intersect(&self, ray: Ray) -> Result<Intersection, ()> {
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

fn main() {
    const W: u32 = 1280;
    const H: u32 = 720;

    let mut img: RgbImage = ImageBuffer::new(W, H);

    let mat_r = Material {
        color: Rgb([255, 0, 0]),
    };
    let mat_g = Material {
        color: Rgb([0, 255, 0]),
    };
    let mat_y = Material {
        color: Rgb([255, 255, 0]),
    };
    let mat_w = Material {
        color: Rgb([255, 255, 255]),
    };

    let s1 = Shape::new(
        mat_r,
        Transform {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion {
                v: Vector3::new(0.0, 0.0, 0.0),
                s: 1.0,
            },
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        Mesh::Sphere { radius: 1.0 },
    );
    let s2 = Shape::new(
        mat_g,
        Transform {
            translation: Vector3::new(1.0, 1.0, -1.0),
            rotation: Quaternion {
                v: Vector3::new(0.0, 0.0, 0.0),
                s: 1.0,
            },
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        Mesh::Sphere { radius: 0.8 },
    );
    let c1 = Shape::new(
        mat_y,
        Transform {
            translation: Vector3::new(-1.0, -0.5, 0.0),
            rotation: axis_angle_to_quat(Vector3::new(1.0, 0.0, 0.0), PI / 4.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        Mesh::Cube {
            size: Vector3::new(1.0, 1.0, 1.0),
        },
    );
    let p1 = Shape::new(
        mat_w,
        Transform {
            translation: Vector3::new(0.0, -2.0, 0.0),
            rotation: Quaternion {
                v: Vector3::new(0.0, 0.0, 0.0),
                s: 1.0,
            },
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        Mesh::InfinitePlane,
    );
    let env_shapes = Shape::new(
        Material {
            color: Rgb([0, 0, 0]),
        },
        Transform {
            translation: Vector3::new(0.0, 0.0, -3.0),
            rotation: axis_angle_to_quat(Vector3::new(0.0, 1.0, 0.0), PI / 4.0),
            scale: Vector3::new(0.7, 1.0, 1.0),
        },
        Mesh::CompositeShape {
            shapes: vec![s1, s2, c1],
        },
    );
    let env = Shape::new(
        Material {
            color: Rgb([0, 0, 0]),
        },
        Transform {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion {
                v: Vector3::new(0.0, 0.0, 0.0),
                s: 1.0,
            },
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        Mesh::CompositeShape {
            shapes: vec![p1, env_shapes],
        },
    );
    // let env = s1;
    let to_sun = Vector3::new(1.0, 3.0, 2.0);
    let camera_center = Vector3::new(0.0, 0.0, 1.0);

    println!("size: {} * {}", W, H);
    let start = Instant::now();
    for w in 0..W {
        for h in 0..H {
            let pixel_pos = Vector3::new(
                (w as f32 + 0.5) / H as f32 - 0.5 * W as f32 / H as f32,
                -((h as f32 + 0.5) / H as f32 - 0.5),
                0.0,
            );

            let ray = Ray {
                pos: camera_center,
                dir: (pixel_pos - camera_center).normalize(),
            };

            if let Ok(info) = env.intersect(ray) {
                let sun_pos = info.pos;
                let sun_ray = Ray {
                    pos: sun_pos + 0.00001 * to_sun,
                    dir: to_sun,
                };
                if let Err(_) = env.intersect(sun_ray) {
                    let intensity = Vector3::angle(info.normal, to_sun).0.cos().clamp(0.0, 1.0);
                    img.put_pixel(
                        w,
                        h,
                        Rgb(info.material.color.0.map(|i| (i as f32 * intensity) as u8)),
                    );
                }
            }
        }
    }
    let duration = start.elapsed();
    println!("time: {:?}", duration);

    img.save("asdf.png").unwrap();
}
