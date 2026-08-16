use crate::incremental::Mesh;
use crate::point::Point;
use crate::predicates::orient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Circumcenter {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoronoiEdge {
    pub a: [f64; 2],
    pub b: [f64; 2],
    pub infinite: bool,
}

pub fn circumcenter(a: Point, b: Point, c: Point) -> Option<Circumcenter> {
    let ax = a.x as f64;
    let ay = a.y as f64;
    let bx = b.x as f64;
    let by = b.y as f64;
    let cx = c.x as f64;
    let cy = c.y as f64;
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-18 {
        return None;
    }
    let a2 = ax * ax + ay * ay;
    let b2 = bx * bx + by * by;
    let c2 = cx * cx + cy * cy;
    let ux = (a2 * (by - cy) + b2 * (cy - ay) + c2 * (ay - by)) / d;
    let uy = (a2 * (cx - bx) + b2 * (ax - cx) + c2 * (bx - ax)) / d;
    Some(Circumcenter { x: ux, y: uy })
}

pub fn voronoi_edges(mesh: &Mesh, bbox_pad: f64) -> (Vec<Circumcenter>, Vec<VoronoiEdge>) {
    let centers: Vec<Option<Circumcenter>> = mesh
        .triangles
        .iter()
        .map(|t| {
            circumcenter(
                mesh.points[t.v[0]],
                mesh.points[t.v[1]],
                mesh.points[t.v[2]],
            )
        })
        .collect();

    let mut edge_to_tris: Vec<((usize, usize), Vec<usize>)> = Vec::new();
    for (ti, t) in mesh.triangles.iter().enumerate() {
        for k in 0..3 {
            let a = t.v[k];
            let b = t.v[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some((_, list)) = edge_to_tris.iter_mut().find(|(e, _)| *e == key) {
                list.push(ti);
            } else {
                edge_to_tris.push((key, vec![ti]));
            }
        }
    }

    let (min_x, max_x, min_y, max_y) = bounds_f64(&mesh.points, bbox_pad);
    let mut edges = Vec::new();
    for ((a, b), tris) in edge_to_tris {
        if tris.len() == 2 {
            if let (Some(c0), Some(c1)) = (centers[tris[0]], centers[tris[1]]) {
                edges.push(VoronoiEdge {
                    a: [c0.x, c0.y],
                    b: [c1.x, c1.y],
                    infinite: false,
                });
            }
        } else if tris.len() == 1 {
            if let Some(c0) = centers[tris[0]] {
                let t = mesh.triangles[tris[0]];
                let third = *t.v.iter().find(|&&v| v != a && v != b).unwrap();
                let pa = mesh.points[a];
                let pb = mesh.points[b];
                let mid = [(pa.x + pb.x) as f64 / 2.0, (pa.y + pb.y) as f64 / 2.0];
                let mut dx = (pb.y - pa.y) as f64;
                let mut dy = (pa.x - pb.x) as f64;
                let len = (dx * dx + dy * dy).sqrt().max(1e-9);
                dx /= len;
                dy /= len;
                let out = Point::new((c0.x + dx * 10.0) as i64, (c0.y + dy * 10.0) as i64);
                if orient(pa, pb, mesh.points[third]) * orient(pa, pb, out) > 0 {
                    dx = -dx;
                    dy = -dy;
                }
                let far = clip_ray(c0.x, c0.y, dx, dy, [min_x, max_x, min_y, max_y]);
                let _ = mid;
                edges.push(VoronoiEdge {
                    a: [c0.x, c0.y],
                    b: far,
                    infinite: true,
                });
            }
        }
    }

    let finite_centers = centers.into_iter().flatten().collect();
    (finite_centers, edges)
}

fn bounds_f64(points: &[Point], pad: f64) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for p in points {
        min_x = min_x.min(p.x as f64);
        max_x = max_x.max(p.x as f64);
        min_y = min_y.min(p.y as f64);
        max_y = max_y.max(p.y as f64);
    }
    if !min_x.is_finite() {
        return (-1.0, 1.0, -1.0, 1.0);
    }
    (min_x - pad, max_x + pad, min_y - pad, max_y + pad)
}

fn clip_ray(x: f64, y: f64, dx: f64, dy: f64, boxxy: [f64; 4]) -> [f64; 2] {
    let [min_x, max_x, min_y, max_y] = boxxy;
    let mut t = f64::MAX;
    if dx > 1e-12 {
        t = t.min((max_x - x) / dx);
    } else if dx < -1e-12 {
        t = t.min((min_x - x) / dx);
    }
    if dy > 1e-12 {
        t = t.min((max_y - y) / dy);
    } else if dy < -1e-12 {
        t = t.min((min_y - y) / dy);
    }
    if !t.is_finite() || t < 0.0 {
        t = (max_x - min_x + max_y - min_y).max(1.0);
    }
    [x + dx * t, y + dy * t]
}
