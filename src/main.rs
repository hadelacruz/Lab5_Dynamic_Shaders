mod framebuffer;
mod vertex;
mod shaders;
mod obj_loader;
mod rasterizer;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use std::f32::consts::PI;

use framebuffer::Framebuffer;
use vertex::Uniforms;
use obj_loader::load_obj;
use shaders::vertex_shader;
use rasterizer::rasterize_triangle;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const FOV: f32 = 45.0 * PI / 180.0;

fn main() {
    let mut window = Window::new(
        "Lab 5: Sol Dinámico con Shaders",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    let (vertices, indices) = load_obj("src/sphere.obj");
    
    let mut camera_distance = 3.5f32;
    let mut camera_angle_x = 0.0f32;
    let mut camera_angle_y = 0.0f32;
    let mut animation_paused = false;
    
    let mut time = 0.0f32;
    let mut frame_count = 0;
    let mut first_frame = true;
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Left) {
            camera_angle_x -= 0.05;
        }
        if window.is_key_down(Key::Right) {
            camera_angle_x += 0.05;
        }
        
        if window.is_key_down(Key::Up) {
            camera_angle_y = (camera_angle_y - 0.05).clamp(-1.5, 1.5);
        }
        if window.is_key_down(Key::Down) {
            camera_angle_y = (camera_angle_y + 0.05).clamp(-1.5, 1.5);
        }
        
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::W) {
            camera_distance = (camera_distance - 0.05).max(1.5);
        }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::S) {
            camera_distance = (camera_distance + 0.05).min(10.0);
        }
        
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            animation_paused = !animation_paused;
            println!("Animación: {}", if animation_paused { "PAUSADA" } else { "ACTIVA" });
        }
        
        if !animation_paused {
            time += 0.025;
        }
        frame_count += 1;
        
        framebuffer.clear(0x000000);
        
        let eye = Vec3::new(
            camera_distance * camera_angle_y.cos() * camera_angle_x.sin(),
            camera_distance * camera_angle_y.sin(),
            camera_distance * camera_angle_y.cos() * camera_angle_x.cos(),
        );
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        let model_matrix = nalgebra_glm::rotate(
            &Mat4::identity(),
            time * 0.2,
            &Vec3::new(0.0, 1.0, 0.0)
        );
        
        let view_matrix = look_at(&eye, &center, &up);
        let projection_matrix = perspective(
            WIDTH as f32 / HEIGHT as f32,
            FOV,
            0.1,
            100.0
        );
        
        let uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            time,
            noise_seed: 42,
        };
        
        for triangle in &indices {
            let v0 = vertex_shader(&vertices[triangle[0]], &uniforms);
            let v1 = vertex_shader(&vertices[triangle[1]], &uniforms);
            let v2 = vertex_shader(&vertices[triangle[2]], &uniforms);
            
            rasterize_triangle(&mut framebuffer, &v0, &v1, &v2, &uniforms);
        }
        
        if first_frame {
            let pixel_count = framebuffer.get_buffer().iter().filter(|&&p| p != 0).count();
            println!("✓ Primer frame: {} píxeles renderizados", pixel_count);
            first_frame = false;
        }
        
        window
            .update_with_buffer(framebuffer.get_buffer(), WIDTH, HEIGHT)
            .unwrap();
        
        if frame_count % 180 == 0 {
            println!("Frame {}: tiempo = {:.2}s (ciclo continuo)", frame_count, time);
        }
    }
    
    println!("\n¡Renderizado finalizado!");
}
