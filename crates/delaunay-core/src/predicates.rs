use crate::point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Left,
    Right,
    Collinear,
}

/// Signed twice-area of triangle abc. Positive is counter-clockwise.
pub fn orient(a: Point, b: Point, c: Point) -> i128 {
    let abx = b.x as i128 - a.x as i128;
    let aby = b.y as i128 - a.y as i128;
    let acx = c.x as i128 - a.x as i128;
    let acy = c.y as i128 - a.y as i128;
    abx * acy - aby * acx
}

pub fn turn(a: Point, b: Point, c: Point) -> Turn {
    match orient(a, b, c).cmp(&0) {
        core::cmp::Ordering::Greater => Turn::Left,
        core::cmp::Ordering::Less => Turn::Right,
        core::cmp::Ordering::Equal => Turn::Collinear,
    }
}

/// InCircle test. If abc is CCW, a positive value means `d` lies inside
/// the circumcircle of abc.
pub fn incircle(a: Point, b: Point, c: Point, d: Point) -> i128 {
    let adx = a.x as i128 - d.x as i128;
    let ady = a.y as i128 - d.y as i128;
    let bdx = b.x as i128 - d.x as i128;
    let bdy = b.y as i128 - d.y as i128;
    let cdx = c.x as i128 - d.x as i128;
    let cdy = c.y as i128 - d.y as i128;
    let abdet = adx * bdy - bdx * ady;
    let bcdet = bdx * cdy - cdx * bdy;
    let cadet = cdx * ady - adx * cdy;
    let alift = adx * adx + ady * ady;
    let blift = bdx * bdx + bdy * bdy;
    let clift = cdx * cdx + cdy * cdy;
    alift * bcdet + blift * cadet + clift * abdet
}
