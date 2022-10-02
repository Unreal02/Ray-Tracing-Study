use std::{fs::File, io::Read};

use crate::*;

pub fn read_obj(name: String) -> Object {
    let mut file = File::open(format!("resources/models/{}.obj", name)).expect("file not found");
    let mut obj = Object::new();

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("something wrong while reading file");

    let lines = content.split("\n");
    for line in lines {
        let split: Vec<&str> = line.split_whitespace().collect();
        match split[0] {
            "v" => {
                obj.points.push(Vec3::new(
                    split[1].parse::<f32>().unwrap(),
                    split[2].parse::<f32>().unwrap(),
                    split[3].parse::<f32>().unwrap(),
                ));
            }
            "vn" => {
                obj.normals.push(Vec3::new(
                    split[1].parse::<f32>().unwrap(),
                    split[2].parse::<f32>().unwrap(),
                    split[3].parse::<f32>().unwrap(),
                ));
            }
            "f" => {
                obj.faces.push(
                    (1..4)
                        .map(|i| {
                            let p: Vec<&str> = split[i].split("/").collect();
                            (
                                p[0].parse::<usize>().unwrap() - 1,
                                p[2].parse::<usize>().unwrap() - 1,
                            )
                        })
                        .collect(),
                );
            }
            _ => {}
        }
    }

    obj
}
