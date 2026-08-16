use crate::hull::convex_hull;
use crate::point::Point;
use crate::predicates::{incircle, orient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle {
    pub v: [usize; 3],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub points: Vec<Point>,
    pub triangles: Vec<Triangle>,
    pub hull: Vec<Point>,
}

/// Bowyer-Watson incremental insertion. Super-triangle vertices are stripped
/// from the returned mesh.
pub fn triangulate(input: &[Point]) -> Mesh {
    let mut unique = Vec::new();
    for &p in input {
        if !unique.contains(&p) {
            unique.push(p);
        }
    }
    if unique.len() < 3 {
        return Mesh {
            hull: convex_hull(&unique),
            points: unique,
            triangles: Vec::new(),
        };
    }

    let (min_x, max_x, min_y, max_y) = bbox(&unique);
    let dx = (max_x - min_x).max(1);
    let dy = (max_y - min_y).max(1);
    let d = dx.max(dy);
    let mid_x = (min_x + max_x) / 2;
    let mid_y = (min_y + max_y) / 2;
    let s1 = Point::new(mid_x - 20 * d, mid_y - d);
    let s2 = Point::new(mid_x + 20 * d, mid_y - d);
    let s3 = Point::new(mid_x, mid_y + 20 * d);

    let mut points = unique.clone();
    let i1 = points.len();
    let i2 = i1 + 1;
    let i3 = i1 + 2;
    points.push(s1);
    points.push(s2);
    points.push(s3);

    let mut tris = vec![ccw_tri(&points, i1, i2, i3)];

    for pi in 0..unique.len() {
        let p = points[pi];
        let mut bad = Vec::new();
        for (ti, t) in tris.iter().enumerate() {
            if incircle(points[t.v[0]], points[t.v[1]], points[t.v[2]], p) > 0 {
                bad.push(ti);
            }
        }
        let mut edge_count: Vec<((usize, usize), i32)> = Vec::new();
        for &ti in &bad {
            let t = tris[ti];
            for k in 0..3 {
                let a = t.v[k];
                let b = t.v[(k + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };
                if let Some(slot) = edge_count.iter_mut().find(|(e, _)| *e == key) {
                    slot.1 += 1;
                } else {
                    edge_count.push((key, 1));
                }
            }
        }
        let keep: Vec<Triangle> = tris
            .iter()
            .enumerate()
            .filter(|(i, _)| !bad.contains(i))
            .map(|(_, t)| *t)
            .collect();
        let mut next = keep;
        for ((a, b), n) in edge_count {
            if n != 1 {
                continue;
            }
            if orient(points[a], points[b], p) == 0 {
                continue;
            }
            next.push(ccw_tri(&points, a, b, pi));
        }
        tris = next;
    }

    let triangles: Vec<Triangle> = tris
        .into_iter()
        .filter(|t| t.v.iter().all(|&v| v < unique.len()))
        .collect();

    Mesh {
        hull: convex_hull(&unique),
        points: unique,
        triangles,
    }
}

fn ccw_tri(points: &[Point], a: usize, b: usize, c: usize) -> Triangle {
    if orient(points[a], points[b], points[c]) < 0 {
        Triangle { v: [a, c, b] }
    } else {
        Triangle { v: [a, b, c] }
    }
}

fn bbox(points: &[Point]) -> (i64, i64, i64, i64) {
    let mut min_x = i64::MAX;
    let mut max_x = i64::MIN;
    let mut min_y = i64::MAX;
    let mut max_y = i64::MIN;
    for p in points {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    (min_x, max_x, min_y, max_y)
}
