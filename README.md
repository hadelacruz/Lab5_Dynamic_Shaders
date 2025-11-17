# Lab 5: Sol Dinámico con Shaders

Simulación de un **sol** con animaciones procedurales usando shaders y funciones de ruido.

## 🎥 Demostración en Video

## https://youtu.be/yyMIBHRIrkM ##

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
- ✅ Modificación solo por shaders
- ✅ Animación continua y cíclica


