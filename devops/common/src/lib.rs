use serde::Serialize;

#[derive(Serialize)]
pub struct STLResult {
    pub valid: bool,
    pub manifoldness_valid: bool,
    pub normals_valid: bool,
    pub triangles_valid: bool,

    pub message: String
}