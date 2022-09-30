use cgmath::{InnerSpace, Matrix, Matrix4, Quaternion, SquareMatrix, Vector3, Vector4};
use image::{ImageBuffer, Rgb, RgbImage};
use std::f32::consts::PI;
use std::time::Instant;

mod shape;
mod transform;

use crate::shape::*;
use crate::transform::*;

fn v4_to_v3(v4: Vector4<f32>) -> Vector3<f32> {
    Vector3::new(v4.x, v4.y, v4.z)
}

fn v3_to_v4(v3: Vector3<f32>, w: f32) -> Vector4<f32> {
    Vector4::new(v3.x, v3.y, v3.z, w)
}

#[derive(Clone, Copy)]
pub struct Material {
    color: Rgb<u8>,
}

#[derive(Clone, Copy)]
pub struct Ray {
    pos: Vector3<f32>,
    dir: Vector3<f32>,
}

#[derive(Clone)]
pub struct Intersection {
    t: f32,
    pos: Vector3<f32>,
    normal: Vector3<f32>,
    local_frame: Matrix4<f32>,
    material: Material,
}

fn axis_angle_to_quat(axis: Vector3<f32>, angle: f32) -> Quaternion<f32> {
    Quaternion {
        v: axis * (angle / 2.0).sin(),
        s: (angle / 2.0).cos(),
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

    let s1 = Shape::new(mat_r, Transform::default(), Mesh::Sphere { radius: 1.0 });
    let s2 = Shape::new(
        mat_g,
        Transform::from_t(Vector3::new(1.0, 1.0, -1.0)),
        Mesh::Sphere { radius: 0.8 },
    );
    let c1 = Shape::new(
        mat_y,
        Transform::from_tr(
            Vector3::new(-1.0, -0.5, 0.0),
            axis_angle_to_quat(Vector3::new(1.0, 0.0, 0.0), PI / 4.0),
        ),
        Mesh::Cube {
            size: Vector3::new(1.0, 1.0, 1.0),
        },
    );
    let p1 = Shape::new(
        mat_w,
        Transform::from_t(Vector3::new(0.0, -2.0, 0.0)),
        Mesh::InfinitePlane,
    );
    let env_shapes = Shape::new(
        Material {
            color: Rgb([0, 0, 0]),
        },
        Transform::from_trs(
            Vector3::new(0.0, 0.0, -3.0),
            axis_angle_to_quat(Vector3::new(0.0, 1.0, 0.0), PI / 4.0),
            Vector3::new(0.7, 1.0, 1.0),
        ),
        Mesh::CompositeShape {
            shapes: vec![s1, s2, c1],
        },
    );
    let env = Shape::new(
        Material {
            color: Rgb([0, 0, 0]),
        },
        Transform::default(),
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
