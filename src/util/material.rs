use crate::*;

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
