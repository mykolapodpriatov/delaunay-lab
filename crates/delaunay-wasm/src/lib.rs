use delaunay_core::{generate, triangulate, voronoi_edges, Distribution, Point};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct MeshJson {
    points: Vec<Point>,
    triangles: Vec<[usize; 3]>,
    hull: Vec<Point>,
    circumcenters: Vec<[f64; 2]>,
    voronoi: Vec<[[f64; 2]; 2]>,
}

#[wasm_bindgen]
pub fn generate_points(seed: u64, distribution: u8, n: usize) -> Result<JsValue, JsValue> {
    let dist = match distribution {
        0 => Distribution::Uniform,
        1 => Distribution::Circle,
        2 => Distribution::Clusters,
        _ => return Err(JsValue::from_str("unknown distribution")),
    };
    let pts = generate(seed, dist, n, 800);
    serde_wasm_bindgen::to_value(&pts).map_err(|e| e.into())
}

#[wasm_bindgen]
pub fn build_mesh(points: JsValue) -> Result<JsValue, JsValue> {
    let pts: Vec<Point> = serde_wasm_bindgen::from_value(points)?;
    let mesh = triangulate(&pts);
    let (centers, edges) = voronoi_edges(&mesh, 80.0);
    let payload = MeshJson {
        points: mesh.points,
        triangles: mesh.triangles.iter().map(|t| t.v).collect(),
        hull: mesh.hull,
        circumcenters: centers.into_iter().map(|c| [c.x, c.y]).collect(),
        voronoi: edges.into_iter().map(|e| [e.a, e.b]).collect(),
    };
    serde_wasm_bindgen::to_value(&payload).map_err(|e| e.into())
}
