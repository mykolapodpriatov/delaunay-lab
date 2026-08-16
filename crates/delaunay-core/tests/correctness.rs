use delaunay_core::{
    convex_hull, generate, incircle, orient, triangulate, voronoi_edges, Distribution, Point,
};

fn area2_sum(points: &[Point], tris: &[[usize; 3]]) -> i128 {
    tris.iter()
        .map(|t| orient(points[t[0]], points[t[1]], points[t[2]]).abs())
        .sum()
}

fn hull_area2(hull: &[Point]) -> i128 {
    if hull.len() < 3 {
        return 0;
    }
    let mut acc = 0_i128;
    for i in 0..hull.len() {
        let a = hull[i];
        let b = hull[(i + 1) % hull.len()];
        acc += a.x as i128 * b.y as i128 - b.x as i128 * a.y as i128;
    }
    acc.abs()
}

#[test]
fn empty_and_tiny() {
    let empty = triangulate(&[]);
    assert!(empty.triangles.is_empty());
    let one = triangulate(&[Point::new(0, 0)]);
    assert!(one.triangles.is_empty());
    let two = triangulate(&[Point::new(0, 0), Point::new(4, 0)]);
    assert!(two.triangles.is_empty());
}

#[test]
fn three_points_one_triangle() {
    let pts = [Point::new(0, 0), Point::new(10, 0), Point::new(0, 10)];
    let mesh = triangulate(&pts);
    assert_eq!(mesh.triangles.len(), 1);
    assert!(orient(pts[0], pts[1], pts[2]) > 0 || mesh.triangles[0].v != [0, 1, 2]);
}

#[test]
fn square_two_triangles() {
    let pts = [
        Point::new(0, 0),
        Point::new(10, 0),
        Point::new(10, 10),
        Point::new(0, 10),
    ];
    let mesh = triangulate(&pts);
    assert_eq!(mesh.triangles.len(), 2);
    let tris: Vec<[usize; 3]> = mesh.triangles.iter().map(|t| t.v).collect();
    assert_eq!(area2_sum(&mesh.points, &tris), hull_area2(&mesh.hull));
}

#[test]
fn all_ccw_and_incircle() {
    let pts = generate(7, Distribution::Uniform, 24, 400);
    let mesh = triangulate(&pts);
    assert!(!mesh.triangles.is_empty());
    for t in &mesh.triangles {
        assert!(
            orient(
                mesh.points[t.v[0]],
                mesh.points[t.v[1]],
                mesh.points[t.v[2]]
            ) > 0,
            "triangle not CCW"
        );
        for (i, &p) in mesh.points.iter().enumerate() {
            if t.v.contains(&i) {
                continue;
            }
            assert!(
                incircle(
                    mesh.points[t.v[0]],
                    mesh.points[t.v[1]],
                    mesh.points[t.v[2]],
                    p
                ) <= 0,
                "point {i} inside circumcircle"
            );
        }
    }
}

#[test]
fn area_matches_hull() {
    let pts = generate(3, Distribution::Clusters, 30, 500);
    let mesh = triangulate(&pts);
    let tris: Vec<[usize; 3]> = mesh.triangles.iter().map(|t| t.v).collect();
    assert_eq!(area2_sum(&mesh.points, &tris), hull_area2(&mesh.hull));
}

#[test]
fn circle_hull_is_all_points() {
    let pts = generate(1, Distribution::Circle, 12, 400);
    let mesh = triangulate(&pts);
    let hull = convex_hull(&mesh.points);
    assert_eq!(hull.len(), mesh.points.len());
    assert_eq!(mesh.hull.len(), hull.len());
}

#[test]
fn voronoi_has_edges() {
    let pts = generate(11, Distribution::Uniform, 16, 300);
    let mesh = triangulate(&pts);
    let (centers, edges) = voronoi_edges(&mesh, 40.0);
    assert_eq!(centers.len(), mesh.triangles.len());
    assert!(!edges.is_empty());
}

#[test]
fn duplicates_are_ignored() {
    let pts = [
        Point::new(0, 0),
        Point::new(0, 0),
        Point::new(8, 0),
        Point::new(0, 8),
    ];
    let mesh = triangulate(&pts);
    assert_eq!(mesh.points.len(), 3);
    assert_eq!(mesh.triangles.len(), 1);
}
