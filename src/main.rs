mod shape;
mod transform;
mod util;

use image::{ImageBuffer, Rgb, RgbImage};
use std::f32::consts::PI;
use std::thread;
use std::time::Instant;
use util::mat4::Mat4;
use util::vec3::Vec3;

use crate::shape::*;
use crate::transform::*;
use crate::util::quat::Quat;

const W: u32 = 1280;
const H: u32 = 720;
const THREAD_COUNT: u32 = 8;

#[derive(Clone, Copy)]
pub struct Material {
    color: Rgb<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pos: Vec3,
    dir: Vec3,
}

#[derive(Clone)]
pub struct Intersection {
    t: f32,
    pos: Vec3,
    normal: Vec3,
    local_frame: Mat4,
    material: Material,
}

fn main() {
    if W % THREAD_COUNT as u32 != 0 {
        panic!("W must be divisible by THREAD_COUNT")
    }

    let mut img: RgbImage = ImageBuffer::new(W, H);
    let to_sun = Vec3::new(1.0, 3.0, 2.0);
    let camera_center = Vec3::new(0.0, 0.0, 1.0);

    println!("size: {} * {}", W, H);
    let start = Instant::now();

    let mut handles = vec![];

    for i in 0..THREAD_COUNT {
        handles.push(thread::spawn(move || {
            render(i, to_sun, camera_center, make_env())
        }));
    }

    let mut i = 0;
    for handle in handles {
        let part_img = handle.join().unwrap();

        let w_interval = W / THREAD_COUNT;
        let w_start = i * w_interval;
        for w in 0..w_interval {
            for h in 0..H {
                img.put_pixel(w + w_start, h, *part_img.get_pixel(w, h));
            }
        }

        i += 1;
    }

    img.save("asdf.png").unwrap();

    let duration = start.elapsed();
    println!("time: {:?}", duration);
}

fn make_env() -> Shape {
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
        Transform::from_t(Vec3::new(1.0, 1.0, -1.0)),
        Mesh::Sphere { radius: 0.8 },
    );
    let c1 = Shape::new(
        mat_y,
        Transform::from_tr(
            Vec3::new(-1.0, -0.5, 0.0),
            Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), PI / 4.0),
        ),
        Mesh::Cube {
            size: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    let p1 = Shape::new(
        mat_w,
        Transform::from_t(Vec3::new(0.0, -2.0, 0.0)),
        Mesh::InfinitePlane,
    );
    let env_shapes = Shape::new(
        Material {
            color: Rgb([0, 0, 0]),
        },
        Transform::from_trs(
            Vec3::new(0.0, 0.0, -3.0),
            Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), PI / 4.0),
            Vec3::new(0.7, 1.0, 1.0),
        ),
        Mesh::CompositeShape {
            shapes: vec![s1, s2, c1],
        },
    );
    Shape::new(
        Material {
            color: Rgb([0, 0, 0]),
        },
        Transform::default(),
        Mesh::CompositeShape {
            shapes: vec![p1, env_shapes],
        },
    )
}

fn render(i: u32, to_sun: Vec3, camera_center: Vec3, env: Shape) -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(W / THREAD_COUNT, H);

    let w_interval = W / THREAD_COUNT;
    let w_start = i * w_interval;
    for w in 0..w_interval {
        for h in 0..H {
            let pixel_pos = Vec3::new(
                ((w + w_start) as f32 + 0.5) / H as f32 - 0.5 * W as f32 / H as f32,
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
                    let intensity = info.normal.angle(to_sun).cos().clamp(0.0, 1.0);
                    img.put_pixel(
                        w,
                        h,
                        Rgb(info.material.color.0.map(|i| (i as f32 * intensity) as u8)),
                    );
                }
            }
        }
    }

    img
}
