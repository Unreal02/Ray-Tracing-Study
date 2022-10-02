use crate::*;

pub struct Object {
    pub points: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<Vec<(usize, usize)>>,
}

impl Object {
    pub fn new() -> Object {
        Object {
            points: vec![],
            normals: vec![],
            faces: vec![],
        }
    }
}
