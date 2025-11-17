use nalgebra_glm::{Vec3, Vec4, Mat4};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use crate::vertex::{Vertex, Uniforms, FragmentInput};

pub fn perlin_noise(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(seed));
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise.set_frequency(Some(2.0));
    noise.get_noise_3d(x, y, z)
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> FragmentInput {
    let wave = (vertex.position.x * 2.0 + uniforms.time * 1.0).sin() * 
               (vertex.position.y * 2.0 + uniforms.time * 0.8).cos();
    
    let displacement = wave * 0.04;
    
    let displaced_pos = Vec3::new(
        vertex.position.x + vertex.normal.x * displacement,
        vertex.position.y + vertex.normal.y * displacement,
        vertex.position.z + vertex.normal.z * displacement,
    );
    
    let displaced_vec4 = Vec4::new(displaced_pos.x, displaced_pos.y, displaced_pos.z, 1.0);
    
    let world_pos = uniforms.model_matrix * displaced_vec4;
    let view_pos = uniforms.view_matrix * world_pos;
    let clip_pos = uniforms.projection_matrix * view_pos;
    
    let model_mat3 = Mat4::identity();
    let normal_matrix = model_mat3.fixed_view::<3, 3>(0, 0).clone();
    let transformed_normal = (normal_matrix * vertex.normal).normalize();
    
    FragmentInput {
        position: clip_pos,
        world_pos: Vec3::new(world_pos.x, world_pos.y, world_pos.z),
        normal: transformed_normal,
        depth: clip_pos.z / clip_pos.w,
    }
}

pub fn fragment_shader(fragment: &FragmentInput, uniforms: &Uniforms) -> u32 {
    let pos = fragment.world_pos;
    let time = uniforms.time;
    let seed = uniforms.noise_seed;
    
    let noise = perlin_noise(pos.x * 1.2, pos.y * 1.2, pos.z * 1.2 + time * 0.3, seed);
    
    let pulse = (time * 1.2).sin() * 0.25 + 0.75;
    
    let flare_wave = ((pos.x + pos.y) * 2.0 + time * 2.0).sin();
    let flare_intensity = (flare_wave * 0.5 + 0.5) * 0.3;
    
    let spot_rotation = (time * 0.8).cos();
    let rotating_spots = ((pos.x * spot_rotation - pos.z * spot_rotation.sin()) * 2.0).sin() * 0.25;
    
    let energy_spike = ((time * 1.8).sin() * (time * 1.3).cos()).abs() * 0.2;
    
    let base_intensity = (noise + 1.0) * 0.5;
    
    let mut intensity = base_intensity * pulse + flare_intensity + energy_spike + rotating_spots;
    intensity = intensity.clamp(0.0, 1.2);
    
    let temperature = intensity;
    
    let color = if temperature < 0.3 {
        let t = temperature / 0.3;
        lerp_color(0x662200, 0xFF4400, t)
    } else if temperature < 0.7 {
        let t = (temperature - 0.3) / 0.4;
        lerp_color(0xFF4400, 0xFFCC00, t)
    } else if temperature < 1.0 {
        let t = (temperature - 0.7) / 0.3;
        lerp_color(0xFFCC00, 0xFFFF66, t)
    } else {
        let t = (temperature - 1.0) / 0.3;
        lerp_color(0xFFFF66, 0xFFFFFF, t.min(1.0))
    };
    
    let emission = if intensity > 0.75 {
        let glow = ((intensity - 0.75) / 0.25).powf(1.5);
        brighten_color(color, glow * 0.4)
    } else {
        color
    };
    
    emission
}

fn lerp_color(c1: u32, c2: u32, t: f32) -> u32 {
    let r1 = ((c1 >> 16) & 0xFF) as f32;
    let g1 = ((c1 >> 8) & 0xFF) as f32;
    let b1 = (c1 & 0xFF) as f32;
    
    let r2 = ((c2 >> 16) & 0xFF) as f32;
    let g2 = ((c2 >> 8) & 0xFF) as f32;
    let b2 = (c2 & 0xFF) as f32;
    
    let r = (r1 + (r2 - r1) * t).min(255.0) as u32;
    let g = (g1 + (g2 - g1) * t).min(255.0) as u32;
    let b = (b1 + (b2 - b1) * t).min(255.0) as u32;
    
    (r << 16) | (g << 8) | b
}

fn brighten_color(color: u32, amount: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32;
    let g = ((color >> 8) & 0xFF) as f32;
    let b = (color & 0xFF) as f32;
    
    let r = (r + 255.0 * amount).min(255.0) as u32;
    let g = (g + 255.0 * amount).min(255.0) as u32;
    let b = (b + 255.0 * amount).min(255.0) as u32;
    
    (r << 16) | (g << 8) | b
}
