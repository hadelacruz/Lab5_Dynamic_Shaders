use nalgebra_glm::{Vec3, Vec4, Mat4};

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Vertex { position, normal }
    }
}

pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub time: f32,
    pub noise_seed: i32,
}

pub struct FragmentInput {
    pub position: Vec4,
    pub world_pos: Vec3,
    pub normal: Vec3,
    pub depth: f32,
}
