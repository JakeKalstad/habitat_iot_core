use std::{env, fs::OpenOptions};
use serde::Serialize;
use stl_io::{IndexedMesh, IndexedTriangle};
use std::collections::HashMap;

fn check_manifold(mesh: &IndexedMesh) -> bool {
    // Simple check: every edge must be shared by exactly two faces
    let mut edge_count = HashMap::new();
    
    for triangle in &mesh.faces {
        let vertices = &triangle.vertices;
        let edges = [
            (vertices[0], vertices[1]),
            (vertices[1], vertices[2]),
            (vertices[2], vertices[0]),
        ];

        for edge in edges.iter() {
            let edge = if edge.0 < edge.1 { edge } else { &(edge.1, edge.0) }; // order edges consistently
            *edge_count.entry(*edge).or_insert(0) += 1;
        }
    }

    // Ensure that every edge is used exactly twice
    edge_count.values().all(|&count| count == 2)
}

fn check_normals(mesh: &IndexedMesh) -> bool {
    // Ensure the normals are oriented correctly (this is a basic check)
    for triangle in &mesh.faces {
        let normal = &triangle.normal;
        // Check if the normal is valid (should not be zero or NaN)
        if normal[0].is_nan() || normal[1].is_nan() || normal[2].is_nan() {
            return false;
        }
    }
    true
}

fn check_valid_triangles(mesh: &IndexedMesh) -> bool {
    // Ensure no degenerate triangles (i.e., all three vertices must be distinct in position)
    for triangle in &mesh.faces {
        let v0 = &mesh.vertices[triangle.vertices[0]];
        let v1 = &mesh.vertices[triangle.vertices[1]];
        let v2 = &mesh.vertices[triangle.vertices[2]];

        // Check if any two vertices have the same position
        if v0 == v1 || v1 == v2 || v0 == v2 {
            return false;
        }
    }
    true
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let stl_file = &args[1];
    println!("Loading STL file: {stl_file}");

    let mut file = OpenOptions::new().read(true).open(stl_file).unwrap();
    let stl = stl_io::read_stl(&mut file).unwrap();
 
    fn write_json(data: common::STLResult) {
        match serde_json::to_string(&data) {
            Ok(json_str) => {
                println!("{}", json_str);  // Print the JSON string to stdout
            },
            Err(e) => {
                eprintln!("Error serializing data to JSON: {}", e);
            }
        }
    }
    if !check_manifold(&stl) {
        write_json(common::STLResult {
            valid: false,
            manifoldness_valid: false,
            normals_valid: true,
            triangles_valid: true,
            message: String::from("Error: The mesh is not manifold (it has holes or edges shared by more than two faces)."),
        });
        return;
    }

    // Check normals
    if !check_normals(&stl) {
        write_json(common::STLResult {
            valid: false,
            manifoldness_valid: true,
            normals_valid: false,
            triangles_valid: true,
            message: String::from("Error: The normals are not correctly defined."),
        });
        return;
    }

    // Check for valid triangles
    if !check_valid_triangles(&stl) {
        write_json(common::STLResult {
            valid: false,
            manifoldness_valid: true,
            normals_valid: true,
            triangles_valid: false,
            message: String::from("Error: The mesh contains degenerate triangles."),
        });
        return;
    }

    // If all checks pass
    write_json(common::STLResult {
        valid: true,
        manifoldness_valid: true,
        normals_valid: true,
        triangles_valid: true,
        message: String::from("The STL mesh is valid and ready for 3D printing."),
    });
}


#[cfg(test)]
mod tests {
    use super::*;
    use stl_io::Vector;

    /// Helper function to generate a valid IndexedMesh
    fn get_valid_mesh() -> IndexedMesh {
        IndexedMesh {
            vertices: vec![
                Vector::new([0.0, 0.0, 0.0]), // Vertex 0
                Vector::new([1.0, 0.0, 0.0]), // Vertex 1
                Vector::new([0.0, 1.0, 0.0]), // Vertex 2
                Vector::new([0.0, 0.0, 1.0]), // Vertex 3
            ],
            faces: vec![
                IndexedTriangle {
                    vertices: [0, 1, 2], // Triangle 1: valid face
                    normal: Vector::new([0.0, 0.0, 1.0]), // Normal facing Z+
                },
                IndexedTriangle {
                    vertices: [0, 1, 3], // Triangle 2: valid face
                    normal: Vector::new([0.0, 1.0, 0.0]), // Normal facing Y+
                },
            ],
        }
    }

    /// Helper function to generate an invalid IndexedMesh
    fn get_invalid_mesh() -> IndexedMesh {
        IndexedMesh {
            vertices: vec![
                Vector::new([0.0, 0.0, 0.0]), // Vertex 0
                Vector::new([0.0, 0.0, 0.0]), // Duplicate Vertex 1
                Vector::new([0.0, 0.0, 0.0]), // Duplicate Vertex 2
            ],
            faces: vec![
                IndexedTriangle {
                    vertices: [0, 1, 2], // Degenerate triangle (all vertices are the same)
                    normal: Vector::new([0.0, 0.0, 0.0]), // Invalid normal
                },
            ],
        }
    }

    #[test]
    fn test_valid_mesh() {
        let mesh = get_valid_mesh();

        // Assume `check_valid_triangles` validates triangle correctness
        let result = check_valid_triangles(&mesh);
        assert!(result, "The valid mesh should pass triangle validation");
    }

    #[test]
    fn test_invalid_mesh() {
        let mesh = get_invalid_mesh();

        // Assume `check_valid_triangles` validates triangle correctness
        let result = check_valid_triangles(&mesh);
        assert!(!result, "The invalid mesh should fail triangle validation");
    }
}
