use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{Vec3, Vec4, Mat4, look_at, perspective};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use std::f32::consts::PI;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const FOV: f32 = 45.0 * PI / 180.0;

// Estructura para representar un vértice
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
}

// Estructura para el Framebuffer
struct Framebuffer {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    zbuffer: Vec<f32>,
}

impl Framebuffer {
    fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            zbuffer: vec![f32::INFINITY; width * height],
        }
    }

    fn clear(&mut self, color: u32) {
        self.buffer.fill(color);
        self.zbuffer.fill(f32::INFINITY);
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: u32, depth: f32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            if depth < self.zbuffer[index] {
                self.buffer[index] = color;
                self.zbuffer[index] = depth;
            }
        }
    }
}

// Estructura para los Uniforms del shader
struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: f32,
    noise_seed: i32,
}

// Estructura para la salida del Vertex Shader
struct FragmentInput {
    position: Vec4,
    world_pos: Vec3,
    normal: Vec3,
    depth: f32,
}

// === FUNCIONES DE RUIDO ===

/// Perlin Noise usando FastNoiseLite
fn perlin_noise(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(seed));
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise.set_frequency(Some(2.0)); // Aumentar frecuencia
    noise.get_noise_3d(x, y, z)
}

/// Cellular Noise (Voronoi)
fn cellular_noise(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(seed));
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_frequency(Some(1.5));
    noise.get_noise_3d(x, y, z)
}

/// Turbulence simplificada: solo 2 octavas (OPTIMIZADO)
fn turbulence(pos: Vec3, time: f32, octaves: i32, seed: i32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    
    // Limitar a máximo 2 octavas para rendimiento
    let max_octaves = octaves.min(2);
    
    for _ in 0..max_octaves {
        sum += amplitude * perlin_noise(
            pos.x * frequency + time * 0.3,
            pos.y * frequency,
            pos.z * frequency + time * 0.2,
            seed
        );
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    sum
}

/// FBM simplificado: solo 2 octavas (OPTIMIZADO)
fn fbm(pos: Vec3, time: f32, seed: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    
    // Solo 2 octavas para mejor rendimiento
    for _ in 0..2 {
        value += amplitude * perlin_noise(
            pos.x * frequency + time * 0.1,
            pos.y * frequency,
            pos.z * frequency,
            seed
        );
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    value
}

// === VERTEX SHADER ===

fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> FragmentInput {
    // Desplazamiento MÍNIMO para mejor rendimiento
    // Solo usar seno/coseno (muy rápido) en lugar de ruido
    let wave = (vertex.position.x * 3.0 + uniforms.time * 2.0).sin() * 
               (vertex.position.y * 3.0 + uniforms.time * 1.5).cos();
    
    let displacement = wave * 0.03; // Desplazamiento muy pequeño
    
    let displaced_pos = Vec3::new(
        vertex.position.x + vertex.normal.x * displacement,
        vertex.position.y + vertex.normal.y * displacement,
        vertex.position.z + vertex.normal.z * displacement,
    );
    
    let displaced_vec4 = Vec4::new(displaced_pos.x, displaced_pos.y, displaced_pos.z, 1.0);
    
    // Transformaciones MVP
    let world_pos = uniforms.model_matrix * displaced_vec4;
    let view_pos = uniforms.view_matrix * world_pos;
    let clip_pos = uniforms.projection_matrix * view_pos;
    
    // Transformar normal
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

// === FRAGMENT SHADER ===

fn fragment_shader(fragment: &FragmentInput, uniforms: &Uniforms) -> u32 {
    let pos = fragment.world_pos;
    let time = uniforms.time;
    let seed = uniforms.noise_seed;
    
    // === OPTIMIZACIÓN MÁXIMA: Solo 1 capa de ruido + efectos animados ===
    
    // Ruido base MUY simplificado
    let noise = perlin_noise(pos.x * 1.2, pos.y * 1.2, pos.z * 1.2 + time * 0.5, seed);
    
    // === EMISIÓN VARIABLE - EFECTOS VISIBLES ===
    
    // Pulsación global del sol (latido)
    let pulse = (time * 2.0).sin() * 0.35 + 0.65; // Pulsación más pronunciada
    
    // Llamaradas que se mueven por la superficie
    let flare_wave = ((pos.x + pos.y) * 3.0 + time * 4.0).sin();
    let flare_intensity = (flare_wave * 0.5 + 0.5) * 0.4; // Llamaradas intensas
    
    // Rotación de manchas solares
    let spot_rotation = (time * 1.5).cos();
    let rotating_spots = ((pos.x * spot_rotation - pos.z * spot_rotation.sin()) * 2.5).sin() * 0.3;
    
    // Picos de energía aleatorios
    let energy_spike = ((time * 3.0).sin() * (time * 2.3).cos()).abs() * 0.25;
    
    // === INTENSIDAD CON ANIMACIÓN ===
    
    let base_intensity = (noise + 1.0) * 0.5;
    
    // Combinar todos los efectos animados
    let mut intensity = base_intensity * pulse + flare_intensity + energy_spike + rotating_spots;
    intensity = intensity.clamp(0.0, 1.3); // Permitir sobrebrillo
    
    // === COLORES DEL SOL CON EMISIÓN BRILLANTE ===
    
    let temperature = intensity;
    
    let color = if temperature < 0.3 {
        // Manchas oscuras (naranja oscuro)
        let t = temperature / 0.3;
        lerp_color(0x662200, 0xFF4400, t)
    } else if temperature < 0.7 {
        // Superficie normal (naranja-amarillo)
        let t = (temperature - 0.3) / 0.4;
        lerp_color(0xFF4400, 0xFFCC00, t)
    } else if temperature < 1.0 {
        // Regiones calientes (amarillo brillante)
        let t = (temperature - 0.7) / 0.3;
        lerp_color(0xFFCC00, 0xFFFF66, t)
    } else {
        // PICOS DE ENERGÍA - Blanco brillante
        let t = (temperature - 1.0) / 0.3;
        lerp_color(0xFFFF66, 0xFFFFFF, t.min(1.0))
    };

    
    // === EMISIÓN (BRILLO) ===
    
    let emission = if intensity > 0.75 {
        let glow = ((intensity - 0.75) / 0.25).powf(1.5);
        brighten_color(color, glow * 0.4)
    } else {
        color
    };
    
    emission
}

// === FUNCIONES AUXILIARES DE COLOR ===

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

// === GENERACIÓN DE ESFERA PROCEDURAL PERFECTA ===

fn create_sphere(radius: f32, latitudes: usize, longitudes: usize) -> (Vec<Vertex>, Vec<[usize; 3]>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Crear vértices usando coordenadas esféricas
    for lat in 0..=latitudes {
        let theta = lat as f32 * PI / latitudes as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        
        for lon in 0..=longitudes {
            let phi = lon as f32 * 2.0 * PI / longitudes as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            
            // Posición en la esfera
            let x = radius * sin_theta * cos_phi;
            let y = radius * cos_theta;
            let z = radius * sin_theta * sin_phi;
            
            // Normal (apunta hacia afuera desde el centro)
            let normal = Vec3::new(x, y, z).normalize();
            
            vertices.push(Vertex {
                position: Vec3::new(x, y, z),
                normal,
            });
        }
    }
    
    // Crear índices para los triángulos
    for lat in 0..latitudes {
        for lon in 0..longitudes {
            let first = lat * (longitudes + 1) + lon;
            let second = first + longitudes + 1;
            
            // Triángulo 1
            indices.push([first, second, first + 1]);
            // Triángulo 2
            indices.push([second, second + 1, first + 1]);
        }
    }
    
    (vertices, indices)
}

// === RASTERIZACIÓN ===

fn rasterize_triangle(
    v0: &FragmentInput,
    v1: &FragmentInput,
    v2: &FragmentInput,
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
) {
    // Clipear triángulos fuera del view frustum
    if v0.position.w <= 0.0 || v1.position.w <= 0.0 || v2.position.w <= 0.0 {
        return;
    }
    
    // Dividir por W para obtener NDC
    let ndc0 = Vec3::new(v0.position.x / v0.position.w, v0.position.y / v0.position.w, v0.position.z / v0.position.w);
    let ndc1 = Vec3::new(v1.position.x / v1.position.w, v1.position.y / v1.position.w, v1.position.z / v1.position.w);
    let ndc2 = Vec3::new(v2.position.x / v2.position.w, v2.position.y / v2.position.w, v2.position.z / v2.position.w);
    
    // Clip en NDC space
    if (ndc0.z < -1.0 || ndc0.z > 1.0) && 
       (ndc1.z < -1.0 || ndc1.z > 1.0) && 
       (ndc2.z < -1.0 || ndc2.z > 1.0) {
        return;
    }
    
    // Convertir NDC a coordenadas de pantalla
    let screen0 = Vec3::new(
        (ndc0.x + 1.0) * 0.5 * framebuffer.width as f32,
        (1.0 - ndc0.y) * 0.5 * framebuffer.height as f32,
        ndc0.z
    );
    let screen1 = Vec3::new(
        (ndc1.x + 1.0) * 0.5 * framebuffer.width as f32,
        (1.0 - ndc1.y) * 0.5 * framebuffer.height as f32,
        ndc1.z
    );
    let screen2 = Vec3::new(
        (ndc2.x + 1.0) * 0.5 * framebuffer.width as f32,
        (1.0 - ndc2.y) * 0.5 * framebuffer.height as f32,
        ndc2.z
    );
    
    // Bounding box
    let min_x = screen0.x.min(screen1.x).min(screen2.x).floor().max(0.0) as usize;
    let max_x = screen0.x.max(screen1.x).max(screen2.x).ceil().min(framebuffer.width as f32 - 1.0) as usize;
    let min_y = screen0.y.min(screen1.y).min(screen2.y).floor().max(0.0) as usize;
    let max_y = screen0.y.max(screen1.y).max(screen2.y).ceil().min(framebuffer.height as f32 - 1.0) as usize;
    
    // Rasterizar cada pixel
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            
            // Coordenadas baricéntricas
            let (w0, w1, w2) = barycentric(
                px, py,
                screen0.x, screen0.y,
                screen1.x, screen1.y,
                screen2.x, screen2.y
            );
            
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                // Interpolación de profundidad
                let depth = w0 * screen0.z + w1 * screen1.z + w2 * screen2.z;
                
                // Interpolación de atributos
                let world_pos = v0.world_pos * w0 + v1.world_pos * w1 + v2.world_pos * w2;
                let normal = (v0.normal * w0 + v1.normal * w1 + v2.normal * w2).normalize();
                
                let fragment = FragmentInput {
                    position: Vec4::new(px, py, depth, 1.0),
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

fn barycentric(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32, f32) {
    let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
    let w0 = ((y1 - y2) * (px - x2) + (x2 - x1) * (py - y2)) / denom;
    let w1 = ((y2 - y0) * (px - x2) + (x0 - x2) * (py - y2)) / denom;
    let w2 = 1.0 - w0 - w1;
    (w0, w1, w2)
}

// === FUNCIÓN PRINCIPAL ===

fn main() {
    let mut window = Window::new(
        "Lab 5: Sol Dinámico con Shaders",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // ~60 FPS

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    // Crear esfera procedural perfecta
    // MUY REDUCIDO: 15 latitudes x 30 longitudes = 900 triángulos (MÁXIMO RENDIMIENTO)
    println!("Generando esfera procedural optimizada...");
    let (vertices, indices) = create_sphere(1.0, 15, 30);
    println!("✓ Esfera creada: {} vértices, {} triángulos", vertices.len(), indices.len());
    
    println!("\n╔═══════════════════════════════════════╗");
    println!("║          CONTROLES 3D                 ║");
    println!("╚═══════════════════════════════════════╝");
    println!("  ROTACIÓN:");
    println!("    ← → : Rotar horizontalmente");
    println!("    ↑ ↓ : Rotar verticalmente");
    println!("  ZOOM:");
    println!("    + / W : Acercar cámara");
    println!("    - / S : Alejar cámara");
    println!("  OTROS:");
    println!("    ESPACIO : Pausar/Reanudar animación");
    println!("    ESC : Salir");
    println!("  - Rotación interactiva");
    println!("  - Zoom de cámara");
    
    // Cámara con controles interactivos
    let mut camera_distance = 3.5f32;
    let mut camera_angle_x = 0.0f32; // Rotación horizontal
    let mut camera_angle_y = 0.0f32; // Rotación vertical
    let mut animation_paused = false;
    
    let mut time = 0.0f32;
    let mut frame_count = 0;
    let mut first_frame = true;
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // CONTROLES DE CÁMARA
        
        // Rotación horizontal (izquierda/derecha)
        if window.is_key_down(Key::Left) {
            camera_angle_x -= 0.05;
        }
        if window.is_key_down(Key::Right) {
            camera_angle_x += 0.05;
        }
        
        // Rotación vertical (arriba/abajo)
        if window.is_key_down(Key::Up) {
            camera_angle_y = (camera_angle_y - 0.05).clamp(-1.5, 1.5);
        }
        if window.is_key_down(Key::Down) {
            camera_angle_y = (camera_angle_y + 0.05).clamp(-1.5, 1.5);
        }
        
        // Zoom (acercar/alejar)
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::W) { // + o W
            camera_distance = (camera_distance - 0.05).max(1.5);
        }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::S) { // - o S
            camera_distance = (camera_distance + 0.05).min(10.0);
        }
        
        // Pausar/reanudar animación
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            animation_paused = !animation_paused;
            println!("Animación: {}", if animation_paused { "PAUSADA" } else { "ACTIVA" });
        }
        
        // Actualizar tiempo solo si no está pausado
        if !animation_paused {
            time += 0.05;
        }
        frame_count += 1;
        
        framebuffer.clear(0x000000); // Fondo negro
        
        // Calcular posición de la cámara basada en ángulos y distancia
        let eye = Vec3::new(
            camera_distance * camera_angle_y.cos() * camera_angle_x.sin(),
            camera_distance * camera_angle_y.sin(),
            camera_distance * camera_angle_y.cos() * camera_angle_x.cos(),
        );
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        // Matrices de transformación (sin rotación automática, control manual)
        let model_matrix = nalgebra_glm::rotate(
            &Mat4::identity(),
            time * 0.1, // Rotación lenta para mantener animación
            &Vec3::new(0.0, 1.0, 0.0)
        );
        
        let view_matrix = look_at(&eye, &center, &up);
        let projection_matrix = perspective(
            WIDTH as f32 / HEIGHT as f32,
            FOV,
            0.1,
            100.0
        );
        
        let viewport_matrix = Mat4::new(
            WIDTH as f32 / 2.0, 0.0, 0.0, WIDTH as f32 / 2.0,
            0.0, -(HEIGHT as f32 / 2.0), 0.0, HEIGHT as f32 / 2.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        
        let uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time, // Variable de tiempo para animación
            noise_seed: 42,
        };
        
        // Renderizar todos los triángulos
        for triangle in &indices {
            let v0 = vertex_shader(&vertices[triangle[0]], &uniforms);
            let v1 = vertex_shader(&vertices[triangle[1]], &uniforms);
            let v2 = vertex_shader(&vertices[triangle[2]], &uniforms);
            
            rasterize_triangle(&v0, &v1, &v2, &mut framebuffer, &uniforms);
        }
        
        if first_frame {
            let non_black_pixels = framebuffer.buffer.iter().filter(|&&p| p != 0).count();
            println!("✓ Primer frame: {} píxeles renderizados", non_black_pixels);
            first_frame = false;
        }
        
        if frame_count % 180 == 0 {
            println!("Frame {}: tiempo = {:.2}s (ciclo continuo)", frame_count, time);
        }
        
        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
    
}