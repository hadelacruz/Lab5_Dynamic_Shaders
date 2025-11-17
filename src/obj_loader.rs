use nalgebra_glm::Vec3;
use crate::vertex::Vertex;
use std::path::Path;

pub fn load_obj(path: &str) -> (Vec<Vertex>, Vec<[usize; 3]>) {
    let obj_path = Path::new(path);
    
    println!("Cargando modelo desde: {}", path);
    
    let (models, _) = tobj::load_obj(
        obj_path,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
    ).expect("Error al cargar archivo OBJ");
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    for model in models {
        let mesh = model.mesh;
        
        for i in (0..mesh.positions.len()).step_by(3) {
            let position = Vec3::new(
                mesh.positions[i],
                mesh.positions[i + 1],
                mesh.positions[i + 2],
            );
            
            let normal = if !mesh.normals.is_empty() {
                Vec3::new(
                    mesh.normals[i],
                    mesh.normals[i + 1],
                    mesh.normals[i + 2],
                )
            } else {
                position.normalize()
            };
            
            vertices.push(Vertex::new(position, normal));
        }
        
        for i in (0..mesh.indices.len()).step_by(3) {
            indices.push([
                mesh.indices[i] as usize,
                mesh.indices[i + 1] as usize,
                mesh.indices[i + 2] as usize,
            ]);
        }
    }
    
    println!("✓ Modelo cargado: {} vértices, {} triángulos", vertices.len(), indices.len());
    
    (vertices, indices)
}
