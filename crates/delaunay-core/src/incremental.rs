use std::collections::{HashMap, HashSet};

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

/// A triangle plus the circumcircle bound used to skip the exact `incircle`
/// test for points that obviously cannot be inside it.
///
/// The circumcentre of an integer triangle is rational, so the bound is kept in
/// the scaled form `(d, ux, uy, r2)` where the centre is `(ux/d, uy/d)` and
/// `r2 = (r*d)^2`. Everything stays in integers, so the filter is exact rather
/// than a float approximation that could reject a point it should not.
///
/// `bound` is `None` for a degenerate triangle (`d == 0`) or when the scaled
/// arithmetic would overflow `i128`; in both cases the exact predicate runs
/// unfiltered, which is the current behaviour.
///
/// The bound lives ON the triangle rather than in a parallel vector, so it
/// cannot drift out of sync when triangles are replaced.
#[derive(Debug, Clone, Copy)]
struct Cell {
    tri: Triangle,
    bound: Option<Circumbound>,
}

#[derive(Debug, Clone, Copy)]
struct Circumbound {
    d: i128,
    ux: i128,
    uy: i128,
    r2: i128,
}

impl Circumbound {
    /// Could `p` lie inside this circumcircle?
    ///
    /// Conservative in one direction only: `false` means definitely outside,
    /// `true` means run the exact predicate. Scaling by `d` clears the
    /// denominators, so the comparison is
    /// `(p.x*d - ux)^2 + (p.y*d - uy)^2 > (r*d)^2` in whole integers with no
    /// rounding to be careful about. Any step that would overflow `i128`
    /// answers `true`, which costs an exact test rather than a wrong one.
    fn may_contain(&self, p: Point) -> bool {
        let Some(dx) = (p.x as i128)
            .checked_mul(self.d)
            .and_then(|v| v.checked_sub(self.ux))
        else {
            return true;
        };
        let Some(dy) = (p.y as i128)
            .checked_mul(self.d)
            .and_then(|v| v.checked_sub(self.uy))
        else {
            return true;
        };
        let Some(dist2) = dx
            .checked_mul(dx)
            .and_then(|x2| dy.checked_mul(dy).and_then(|y2| x2.checked_add(y2)))
        else {
            return true;
        };
        dist2 <= self.r2
    }
}

/// The scaled circumcircle of `a`, `b`, `c`, or `None` when it is degenerate or
/// does not fit in `i128`.
fn circumbound(a: Point, b: Point, c: Point) -> Option<Circumbound> {
    let (ax, ay) = (a.x as i128, a.y as i128);
    let (bx, by) = (b.x as i128, b.y as i128);
    let (cx, cy) = (c.x as i128, c.y as i128);

    let d = 2_i128.checked_mul(
        ax.checked_mul(by.checked_sub(cy)?)?
            .checked_add(bx.checked_mul(cy.checked_sub(ay)?)?)?
            .checked_add(cx.checked_mul(ay.checked_sub(by)?)?)?,
    )?;
    if d == 0 {
        return None;
    }

    let a2 = ax.checked_mul(ax)?.checked_add(ay.checked_mul(ay)?)?;
    let b2 = bx.checked_mul(bx)?.checked_add(by.checked_mul(by)?)?;
    let c2 = cx.checked_mul(cx)?.checked_add(cy.checked_mul(cy)?)?;

    let ux = a2
        .checked_mul(by.checked_sub(cy)?)?
        .checked_add(b2.checked_mul(cy.checked_sub(ay)?)?)?
        .checked_add(c2.checked_mul(ay.checked_sub(by)?)?)?;
    let uy = a2
        .checked_mul(cx.checked_sub(bx)?)?
        .checked_add(b2.checked_mul(ax.checked_sub(cx)?)?)?
        .checked_add(c2.checked_mul(bx.checked_sub(ax)?)?)?;

    // (r*d)^2, measured from vertex a.
    let rx = ax.checked_mul(d)?.checked_sub(ux)?;
    let ry = ay.checked_mul(d)?.checked_sub(uy)?;
    let r2 = rx.checked_mul(rx)?.checked_add(ry.checked_mul(ry)?)?;

    Some(Circumbound { d, ux, uy, r2 })
}

/// Bowyer-Watson incremental insertion. Super-triangle vertices are stripped
/// from the returned mesh.
pub fn triangulate(input: &[Point]) -> Mesh {
    // First-seen order is preserved: point indices are part of the output, so
    // reordering them would churn the mesh for no reason.
    let mut seen = HashSet::with_capacity(input.len());
    let mut unique = Vec::with_capacity(input.len());
    for &p in input {
        if seen.insert(p) {
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
    let offset = super_triangle_offset(d);
    let s1 = Point::new(mid_x - offset, mid_y - offset);
    let s2 = Point::new(mid_x + offset, mid_y - offset);
    let s3 = Point::new(mid_x, mid_y + offset);

    let mut points = unique.clone();
    let i1 = points.len();
    let i2 = i1 + 1;
    let i3 = i1 + 2;
    points.push(s1);
    points.push(s2);
    points.push(s3);

    let mut tris = vec![cell(&points, ccw_tri(&points, i1, i2, i3))];

    // Reused across insertions so the per-point allocation does not dominate
    // once the cheap rejection has taken the predicate cost down.
    let mut is_bad: Vec<bool> = Vec::new();
    let mut edge_order: Vec<(usize, usize)> = Vec::new();
    let mut edge_count: HashMap<(usize, usize), i32> = HashMap::new();

    for pi in 0..unique.len() {
        let p = points[pi];

        is_bad.clear();
        is_bad.resize(tris.len(), false);
        let mut any_bad = false;
        for (ti, c) in tris.iter().enumerate() {
            // Cheap rejection first; the exact predicate still decides.
            if let Some(bound) = c.bound {
                if !bound.may_contain(p) {
                    continue;
                }
            }
            let t = c.tri;
            if incircle(points[t.v[0]], points[t.v[1]], points[t.v[2]], p) > 0 {
                is_bad[ti] = true;
                any_bad = true;
            }
        }
        if !any_bad {
            continue;
        }

        // Counted through a map, but emitted in insertion order: HashMap
        // iteration order varies per run and the triangle list is part of the
        // output, so the mesh has to be built from a stable sequence.
        edge_order.clear();
        edge_count.clear();
        for (ti, c) in tris.iter().enumerate() {
            if !is_bad[ti] {
                continue;
            }
            let t = c.tri;
            for k in 0..3 {
                let a = t.v[k];
                let b = t.v[(k + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };
                match edge_count.get_mut(&key) {
                    Some(n) => *n += 1,
                    None => {
                        edge_count.insert(key, 1);
                        edge_order.push(key);
                    }
                }
            }
        }

        let mut index = 0;
        tris.retain(|_| {
            let keep = !is_bad[index];
            index += 1;
            keep
        });

        for &(a, b) in &edge_order {
            if edge_count[&(a, b)] != 1 {
                continue;
            }
            if orient(points[a], points[b], p) == 0 {
                continue;
            }
            tris.push(cell(&points, ccw_tri(&points, a, b, pi)));
        }
    }

    let triangles: Vec<Triangle> = tris
        .into_iter()
        .map(|c| c.tri)
        .filter(|t| t.v.iter().all(|&v| v < unique.len()))
        .collect();

    Mesh {
        hull: convex_hull(&unique),
        points: unique,
        triangles,
    }
}

/// How far each super-triangle vertex must sit from the bounding-box
/// midpoint so that it can never fall inside the circumcircle of a triangle
/// built from real points, i.e. so stripping the super vertices at the end
/// never leaves a hole where a super-vertex-touching triangle "won" over the
/// real one on the boundary.
///
/// Derivation: any two points inside the bounding box are within Euclidean
/// distance `2*d` of each other (each axis span is at most `d`, so each
/// point is within `d` of the midpoint, by the triangle inequality). Points
/// sit on an integer lattice, so a non-degenerate triangle has a twice-area
/// of at least one, meaning by `R = (side_a * side_b * side_c) / (4 * area)`
/// its circumradius is at most `(2d)^3 / 2`, i.e. `4*d^3`. A point further
/// than `2*R + d` from the midpoint is outside that circle: crossing the
/// disk from any point on it costs at most `2*R`, and the circumcenter
/// itself is within `R + d` of the midpoint (it sits exactly `R` from a
/// vertex, which is within `d` of the midpoint). So `8*d^3 + d` is a
/// proven-sufficient offset.
///
/// That bound is only tight for a deliberately near-degenerate triangle
/// (twice-area of exactly one) stretched across the whole point cloud;
/// ordinary point clouds need far less. It also grows faster than the exact
/// `i128` arithmetic in `incircle`/`orient` can take once the super-triangle
/// vertices are folded in: the very first triangle is the three super
/// vertices themselves, and its `incircle` test against the first real point
/// multiplies four offset-scaled terms together, so the offset needs to stay
/// under roughly the fourth root of `i128::MAX`. `SAFE_CEILING` is
/// comfortably inside that limit, so the smaller of the two bounds is used;
/// the proven-sufficient bound only matters, and only applies, for small
/// point clouds, where it is well under the ceiling anyway.
fn super_triangle_offset(d: i64) -> i64 {
    const SAFE_CEILING: i128 = 1_000_000_000;

    let d128 = d as i128;
    let proven_sufficient = d128
        .checked_mul(d128)
        .and_then(|d2| d2.checked_mul(d128))
        .and_then(|d3| d3.checked_mul(8))
        .and_then(|v| v.checked_add(d128))
        .and_then(|v| v.checked_add(1))
        .unwrap_or(i128::MAX);

    proven_sufficient.min(SAFE_CEILING).max(d128 + 1) as i64
}

/// Wrap a triangle with its circumcircle bound.
fn cell(points: &[Point], tri: Triangle) -> Cell {
    let bound = circumbound(points[tri.v[0]], points[tri.v[1]], points[tri.v[2]]);
    Cell { tri, bound }
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
