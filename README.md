# Lab 5: Sol Dinámico con Shaders

Simulación de un **sol** con animaciones procedurales usando shaders y funciones de ruido.

## 🎥 Demostración en Video


**https://youtu.be/QfwNnYGkfGQ**

---

## 🎯 Características Implementadas

### ✅ Criterios Cumplidos

| Criterio | Implementación |
|----------|----------------|
| **Complejidad del shader** | ✅ Perlin noise + 5 efectos matemáticos combinados |
| **Animación continua** | ✅ Variable `time` con múltiples frecuencias (2x, 4x, 1.5x, 3x) |
| **Perlin noise ajustable** | ✅ FastNoiseLite con frequency=2.0, seed y coordenadas 3D |
| **Emisión variable** | ✅ Pulsación, llamaradas, picos de energía, manchas rotatorias |
| **Gradiente dinámico** | ✅ 4 niveles de color basados en temperatura/intensidad |
| **Flare en Vertex Shader** | ✅ Desplazamiento ondulatorio con sin/cos |

### 🚀 Optimizaciones
- **496 vértices, 900 triángulos** (geometría reducida)
- **Vertex shader**: Solo funciones trigonométricas (10x más rápido que ruido)
- **Fragment shader**: 1 capa de Perlin + efectos matemáticos
- **Rendimiento**: ~60 FPS en modo release

---

## 📐 Arquitectura del Shader

### Vertex Shader
**Propósito:** Desplazamiento ondulatorio de la superficie

```rust
fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> FragmentInput {
    // Ondas superficiales usando sin/cos
    let wave = (vertex.position.x * 3.0 + uniforms.time * 2.0).sin() * 
               (vertex.position.y * 3.0 + uniforms.time * 1.5).cos();
    
    let displacement = wave * 0.03;
    displaced_pos = pos + normal * displacement;
}
```

**Parámetros:**
- `time`: Variable de animación continua
- Frecuencias: 3.0 (espacial), 2.0 y 1.5 (temporal)
- Intensidad: 0.03 (desplazamiento sutil)

### Fragment Shader
**Propósito:** Color dinámico con emisión variable

```rust
fn fragment_shader(fragment: &FragmentInput, uniforms: &Uniforms) -> u32 {
    // 1. Ruido base (textura)
    let noise = perlin_noise(pos * 1.2 + time * 0.5);
    
    // 2. Efectos de emisión variable
    let pulse = (time * 2.0).sin() * 0.35 + 0.65;              // Latido global
    let flare = ((pos.x + pos.y) * 3.0 + time * 4.0).sin();    // Llamaradas
    let spots = ((pos.x * cos(time*1.5) - pos.z * sin) * 2.5).sin(); // Manchas
    let spike = ((time*3.0).sin() * (time*2.3).cos()).abs();   // Picos aleatorios
    
    // 3. Combinar efectos
    intensity = noise * pulse + flare + spike + spots;
    
    // 4. Mapeo de color por temperatura
    color = gradient(intensity); // 4 niveles: naranja oscuro → blanco
}
```

**Efectos de Emisión Variable:**
1. **Pulsación Global** (`pulse`): Latido del sol (ciclo 3s)
2. **Llamaradas** (`flare`): Ondas rápidas de energía
3. **Manchas Rotatorias** (`spots`): Áreas oscuras que rotan
4. **Picos Aleatorios** (`spike`): Flashes brillantes

---

## 🔧 Funciones de Ruido

### Perlin Noise
```rust
fn perlin_noise(x: f32, y: f32, z: f32, seed: i32) -> f32
```
- **Librería**: FastNoiseLite
- **Tipo**: NoiseType::Perlin
- **Frecuencia**: 2.0
- **Rango**: [-1.0, 1.0]
- **Uso**: Textura base de la superficie solar

**Parámetros ajustables:**
- `seed`: Control de variación (default: 42)
- `frequency`: Densidad del patrón (default: 2.0)
- `x, y, z`: Coordenadas 3D (z incluye `time` para animación)

---

## 🎮 Uniforms

```rust
struct Uniforms {
    model_matrix: Mat4,      // Rotación del sol
    view_matrix: Mat4,       // Posición de cámara
    projection_matrix: Mat4, // Perspectiva
    time: f32,               // Variable de animación continua
    noise_seed: i32,         // Seed para ruido
}
```

**Variable clave: `time`**
- Incremento: `time += 0.05` (60 FPS)
- Usado en vertex shader para ondas
- Usado en fragment shader con 5 frecuencias diferentes
- Garantiza animación continua y cíclica

---

## 🛠️ Instalación y Uso

### Requisitos
- Rust (edición 2021)
- Cargo

### Dependencias
```toml
minifb = "0.25"           # Ventana y renderizado
nalgebra-glm = "0.18"     # Matemáticas 3D
fastnoise-lite = "1.1"    # Generación de ruido
```

### Ejecutar
```bash
# Modo debug (más lento pero funcional)
cargo run

# Modo release (optimizado, 60 FPS)
cargo run --release
```

**Controles:**
- `ESC`: Salir

---

## 📊 Métricas de Rendimiento

| Métrica | Valor |
|---------|-------|
| Vértices | 496 |
| Triángulos | 900 |
| Resolución | 800×600 |
| Píxeles/frame | ~144,000 |
| FPS (release) | 60 |

---


## 📝 Resumen de Implementación

**Restricciones cumplidas:**
- ✅ Solo geometría esférica
- ✅ Sin texturas ni materiales
- ✅ Animación con `uniform float time`
- ✅ Modificación solo por shaders
- ✅ Animación continua y cíclica

**Técnicas usadas:**
- Perlin noise (FastNoiseLite)
- Desplazamiento de vértices
- Emisión variable (5 efectos)
- Gradiente dinámico de color
- Optimización agresiva para release mode
