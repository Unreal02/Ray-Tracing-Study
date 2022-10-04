mod obj_reader;
mod shape;
mod transform;
mod util;

use image::{ImageBuffer, Rgb, RgbImage};
use std::f32::consts::PI;
use std::thread;
use std::time::Instant;

pub use obj_reader::*;
pub use shape::*;
pub use transform::*;
pub use util::*;

const W: u32 = 1280;
const H: u32 = 720;
const THREAD_COUNT: u32 = 16;

#[derive(Clone, Copy, Debug)]
pub enum Material {
    Simple {
        color: Rgb<u8>,
    },
    Checkerboard {
        color1: Rgb<u8>,
        color2: Rgb<u8>,
        scale: f32,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pos: Vec3,
    dir: Vec3,
}

#[derive(Clone, Debug)]
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
    let mat_s1 = Material::Simple {
        color: Rgb([255, 0, 0]),
    };
    let mat_s2 = Material::Simple {
        color: Rgb([0, 255, 0]),
    };
    let mat_c1 = Material::Checkerboard {
        color1: Rgb([255, 255, 0]),
        color2: Rgb([0, 255, 255]),
        scale: 0.333333,
    };
    let mat_p1 = Material::Checkerboard {
        color1: Rgb([255, 255, 255]),
        color2: Rgb([127, 127, 127]),
        scale: 1.0,
    };
    let mat_p2 = Material::Checkerboard {
        color1: Rgb([0, 255, 255]),
        color2: Rgb([0, 127, 127]),
        scale: 1.0,
    };

    let s1 = Shape::new(mat_s1, Transform::default(), Mesh::Sphere { radius: 1.0 });
    let s2 = Shape::new(
        mat_s2,
        Transform::from_t(Vec3::new(1.0, 1.0, -1.0)),
        Mesh::Sphere { radius: 0.8 },
    );
    let c1 = Shape::new(
        mat_c1,
        Transform::from_tr(
            Vec3::new(-1.0, -0.5, 0.0),
            Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), PI / 4.0),
        ),
        Mesh::Cube {
            size: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    let p1 = Shape::new(
        mat_p1,
        Transform::from_t(Vec3::new(0.0, -2.0, 0.0)),
        Mesh::InfinitePlane,
    );
    let env_shapes = Shape::new(
        Material::Simple {
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
    let env = Shape::new(
        Material::Simple {
            color: Rgb([0, 0, 0]),
        },
        Transform::default(),
        Mesh::CompositeShape {
            shapes: vec![p1, env_shapes],
        },
    );

    let p2 = Shape::new(mat_p2, Transform::default(), Mesh::InfinitePlane);
    let teapot = Shape::new(
        Material::Simple {
            color: Rgb([255, 255, 255]),
        },
        Transform::default(),
        Mesh::Polygons {
            obj: read_obj(String::from("teapot")),
        },
    );
    let env_teapot = Shape::new(
        Material::Simple {
            color: Rgb([0, 0, 0]),
        },
        Transform::from_tr(
            Vec3::new(0.0, -0.8, -6.0),
            Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), PI / 4.0),
        ),
        Mesh::CompositeShape {
            shapes: vec![teapot, p2],
        },
    );

    env
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

            if let Some(info) = env.intersect(ray) {
                let sun_pos = info.pos;
                let sun_ray = Ray {
                    pos: sun_pos + 0.00001 * to_sun,
                    dir: to_sun,
                };
                if let None = env.intersect(sun_ray) {
                    let intensity = info.normal.angle(to_sun).cos().clamp(0.0, 1.0);
                    let color = match info.material {
                        Material::Simple { color } => color,
                        Material::Checkerboard {
                            color1,
                            color2,
                            scale,
                        } => {
                            let local_pos =
                                info.local_frame.invert().unwrap() * Vec4::from_vec3(info.pos, 1.0);
                            if ((local_pos.x / scale).round() as i32
                                + (local_pos.y / scale).round() as i32
                                + (local_pos.z / scale).round() as i32)
                                % 2
                                == 0
                            {
                                color1
                            } else {
                                color2
                            }
                        }
                    };
                    img.put_pixel(w, h, Rgb(color.0.map(|i| (i as f32 * intensity) as u8)));
                }
            }
        }
    }

    img
}
