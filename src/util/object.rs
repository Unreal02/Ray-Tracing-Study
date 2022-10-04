use crate::*;

#[derive(Default)]
pub struct Polygon {
    pub points: Vec<Vec3>,
    pub normals: Vec<Vec3>,
}

pub struct Object {
    pub points: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub polygons: Vec<Polygon>,
}

impl Object {
    pub fn new() -> Object {
        Object {
            points: vec![],
            normals: vec![],
            polygons: vec![],
        }
    }
}
