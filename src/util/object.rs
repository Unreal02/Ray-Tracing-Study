use crate::*;

#[derive(Clone, Default)]
pub struct Polygon {
    pub points: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub e1: Vec3,
    pub e2: Vec3,
    pub polygon_normal: Vec3, // not normalized
}

#[derive(Clone)]
pub struct Object {
    pub polygons: Vec<Polygon>,
    pub bounding_box: (Vec3, Vec3), // (min_coordinate, max_coordinate)
}

impl Object {
    pub fn new() -> Object {
        Object {
            polygons: vec![],
            bounding_box: (
                Vec3::new(1.0 / 0.0, 1.0 / 0.0, 1.0 / 0.0),
                Vec3::new(1.0 / -0.0, 1.0 / -0.0, 1.0 / -0.0),
            ),
        }
    }
}
