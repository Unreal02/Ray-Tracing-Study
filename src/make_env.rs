use crate::*;
use std::f32::consts::PI;

pub enum Env {
    Default,
    Teapot,
}

pub fn make_env(name: Env) -> Shape {
    match name {
        Env::Default => {
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
            Shape::new(
                Material::Simple {
                    color: Rgb([0, 0, 0]),
                },
                Transform::default(),
                Mesh::CompositeShape {
                    shapes: vec![p1, env_shapes],
                },
            )
        }
        Env::Teapot => {
            let mat_p = Material::Checkerboard {
                color1: Rgb([0, 255, 255]),
                color2: Rgb([0, 127, 127]),
                scale: 1.0,
            };
            let mat_teapot = Material::Simple {
                color: Rgb([0, 0, 0]),
            };

            let p = Shape::new(mat_p, Transform::default(), Mesh::InfinitePlane);
            let teapot = Shape::new(
                Material::Simple {
                    color: Rgb([255, 255, 255]),
                },
                Transform::default(),
                Mesh::Polygons {
                    obj: read_obj(String::from("teapot")),
                },
            );
            Shape::new(
                mat_teapot,
                Transform::from_tr(
                    Vec3::new(0.0, -0.8, -6.0),
                    Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), PI / 4.0),
                ),
                Mesh::CompositeShape {
                    shapes: vec![teapot, p],
                },
            )
        }
    }
}
