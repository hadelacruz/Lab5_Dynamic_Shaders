use nalgebra_glm::Vec4;
use crate::framebuffer::Framebuffer;
use crate::vertex::{Uniforms, FragmentInput};
use crate::shaders::fragment_shader;

pub fn rasterize_triangle(
    framebuffer: &mut Framebuffer,
    v0: &FragmentInput,
    v1: &FragmentInput,
    v2: &FragmentInput,
    uniforms: &Uniforms,
) {
    let w0 = v0.position.w;
    let w1 = v1.position.w;
    let w2 = v2.position.w;
    
    let ndc0 = v0.position / w0;
    let ndc1 = v1.position / w1;
    let ndc2 = v2.position / w2;
    
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    
    let screen0 = ((ndc0.x + 1.0) * width / 2.0, (1.0 - ndc0.y) * height / 2.0);
    let screen1 = ((ndc1.x + 1.0) * width / 2.0, (1.0 - ndc1.y) * height / 2.0);
    let screen2 = ((ndc2.x + 1.0) * width / 2.0, (1.0 - ndc2.y) * height / 2.0);
    
    let min_x = screen0.0.min(screen1.0).min(screen2.0).max(0.0) as usize;
    let max_x = screen0.0.max(screen1.0).max(screen2.0).min(width - 1.0) as usize;
    let min_y = screen0.1.min(screen1.1).min(screen2.1).max(0.0) as usize;
    let max_y = screen0.1.max(screen1.1).max(screen2.1).min(height - 1.0) as usize;
    
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            
            let (w0, w1, w2) = barycentric(p, screen0, screen1, screen2);
            
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let depth = w0 * ndc0.z + w1 * ndc1.z + w2 * ndc2.z;
                
                let world_pos = v0.world_pos * w0 + v1.world_pos * w1 + v2.world_pos * w2;
                let normal = (v0.normal * w0 + v1.normal * w1 + v2.normal * w2).normalize();
                
                let fragment = FragmentInput {
                    position: Vec4::new(p.0, p.1, depth, 1.0),
                    world_pos,
                    normal,
                    depth,
                };
                
                let color = fragment_shader(&fragment, uniforms);
                framebuffer.set_pixel(x, y, color, depth);
            }
        }
    }
}

fn barycentric(
    p: (f32, f32),
    v0: (f32, f32),
    v1: (f32, f32),
    v2: (f32, f32),
) -> (f32, f32, f32) {
    let denom = (v1.1 - v2.1) * (v0.0 - v2.0) + (v2.0 - v1.0) * (v0.1 - v2.1);
    
    if denom.abs() < 0.0001 {
        return (-1.0, -1.0, -1.0);
    }
    
    let w0 = ((v1.1 - v2.1) * (p.0 - v2.0) + (v2.0 - v1.0) * (p.1 - v2.1)) / denom;
    let w1 = ((v2.1 - v0.1) * (p.0 - v2.0) + (v0.0 - v2.0) * (p.1 - v2.1)) / denom;
    let w2 = 1.0 - w0 - w1;
    
    (w0, w1, w2)
}
