mod make_env;
mod obj_reader;
mod shape;
mod transform;
mod util;

use image::{ImageBuffer, Rgb, RgbImage};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

pub use make_env::*;
pub use obj_reader::*;
pub use shape::*;
pub use transform::*;
pub use util::*;

const W: u32 = 1280;
const H: u32 = 720;
const THREAD_COUNT: u32 = 16;
const CURRENT_ENV: Env = Env::Default;
const SAMPLE_NUMBER: u32 = 16;

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

    let env = make_env(CURRENT_ENV);
    let lock = Arc::new(env);

    let mut handles = vec![];
    for i in 0..THREAD_COUNT {
        let clone_lock = Arc::clone(&lock);
        handles.push(thread::spawn(move || {
            render(i, to_sun, camera_center, clone_lock)
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

fn render(i: u32, to_sun: Vec3, camera_center: Vec3, env: Arc<Shape>) -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(W / THREAD_COUNT, H);

    let w_interval = W / THREAD_COUNT;
    let w_start = i * w_interval;
    for w in 0..w_interval {
        for h in 0..H {
            let mut color_sum = [0.0, 0.0, 0.0];
            for _ in 0..SAMPLE_NUMBER {
                let dx = rand::random::<f32>();
                let dy = rand::random::<f32>();
                let pixel_pos = Vec3::new(
                    ((w + w_start) as f32 + dx) / H as f32 - 0.5 * W as f32 / H as f32,
                    -((h as f32 + dy) / H as f32 - 0.5),
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
                                let local_pos = info.local_frame.invert().unwrap()
                                    * Vec4::from_vec3(info.pos, 1.0);
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
                        for i in 0..3 {
                            color_sum[i] += color[i] as f32 * intensity;
                        }
                    }
                }
            }
            img.put_pixel(
                w,
                h,
                Rgb(color_sum.map(|i| (i / SAMPLE_NUMBER as f32) as u8)),
            );
        }
    }

    img
}
