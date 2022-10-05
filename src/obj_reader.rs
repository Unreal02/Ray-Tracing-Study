use std::{fs::File, io::Read};

use crate::*;

pub fn read_obj(name: String) -> Object {
    let mut file = File::open(format!("resources/models/{}.obj", name)).expect("file not found");
    let mut obj = Object::new();

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("something wrong while reading file");

    let mut points: Vec<Vec3> = vec![];
    let mut normals: Vec<Vec3> = vec![];

    let lines = content.split("\n");
    for line in lines {
        let split: Vec<&str> = line.split_whitespace().collect();
        match split[0] {
            "v" => {
                let point = Vec3::new(
                    split[1].parse::<f32>().unwrap(),
                    split[2].parse::<f32>().unwrap(),
                    split[3].parse::<f32>().unwrap(),
                );
                points.push(point);
                if obj.bounding_box.0.x > point.x {
                    obj.bounding_box.0.x = point.x;
                }
                if obj.bounding_box.0.y > point.y {
                    obj.bounding_box.0.y = point.y;
                }
                if obj.bounding_box.0.z > point.z {
                    obj.bounding_box.0.z = point.z;
                }
                if obj.bounding_box.1.x < point.x {
                    obj.bounding_box.1.x = point.x;
                }
                if obj.bounding_box.1.y < point.y {
                    obj.bounding_box.1.y = point.y;
                }
                if obj.bounding_box.1.z < point.z {
                    obj.bounding_box.1.z = point.z;
                }
            }
            "vn" => {
                normals.push(Vec3::new(
                    split[1].parse::<f32>().unwrap(),
                    split[2].parse::<f32>().unwrap(),
                    split[3].parse::<f32>().unwrap(),
                ));
            }
            "f" => {
                let mut polygon = Polygon::default();
                for i in 1..4 {
                    let p: Vec<&str> = split[i].split("/").collect();
                    polygon
                        .points
                        .push(points[p[0].parse::<usize>().unwrap() - 1]);
                    polygon
                        .normals
                        .push(normals[p[2].parse::<usize>().unwrap() - 1]);
                }

                polygon.e1 = polygon.points[1] - polygon.points[0];
                polygon.e2 = polygon.points[2] - polygon.points[0];
                polygon.polygon_normal = polygon.e1.cross(polygon.e2);
                obj.polygons.push(polygon);
            }
            _ => {}
        }
    }

    obj
}
