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
///
/// The magnitude is the exact lifted determinant when it fits in `i128`.
/// A determinant past that range is saturated to `i128::MAX` or `i128::MIN`,
/// so the sign stays right for coordinates that overflow a bare `i128` product.
pub fn incircle(a: Point, b: Point, c: Point, d: Point) -> i128 {
    let adx = a.x as i128 - d.x as i128;
    let ady = a.y as i128 - d.y as i128;
    let bdx = b.x as i128 - d.x as i128;
    let bdy = b.y as i128 - d.y as i128;
    let cdx = c.x as i128 - d.x as i128;
    let cdy = c.y as i128 - d.y as i128;
    let abdet = det(adx, ady, bdx, bdy);
    let bcdet = det(bdx, bdy, cdx, cdy);
    let cadet = det(cdx, cdy, adx, ady);
    let alift = lift(adx, ady);
    let blift = lift(bdx, bdy);
    let clift = lift(cdx, cdy);
    let sum = abdet.mul(clift).add(bcdet.mul(alift)).add(cadet.mul(blift));
    sum.to_i128()
}

/// 320-bit signed integer. Little-endian magnitude. Enough for the incircle
/// determinant of any `i64` coordinates (about 264 bits).
#[derive(Clone, Copy)]
struct Wide {
    neg: bool,
    mag: [u64; 5],
}

impl Wide {
    fn zero() -> Self {
        Self {
            neg: false,
            mag: [0; 5],
        }
    }

    fn from_i128(v: i128) -> Self {
        if v == 0 {
            return Self::zero();
        }
        let abs = v.unsigned_abs();
        Self {
            neg: v < 0,
            mag: [abs as u64, (abs >> 64) as u64, 0, 0, 0],
        }
    }

    fn is_zero(self) -> bool {
        self.mag.iter().all(|limb| *limb == 0)
    }

    fn add(self, other: Self) -> Self {
        if self.neg == other.neg {
            return Self {
                neg: self.neg && !self.is_zero(),
                mag: add_mag(self.mag, other.mag),
            };
        }
        match cmp_mag(self.mag, other.mag) {
            core::cmp::Ordering::Equal => Self::zero(),
            core::cmp::Ordering::Greater => Self {
                neg: self.neg,
                mag: sub_mag(self.mag, other.mag),
            },
            core::cmp::Ordering::Less => Self {
                neg: other.neg,
                mag: sub_mag(other.mag, self.mag),
            },
        }
    }

    fn mul(self, other: Self) -> Self {
        let mag = mul_mag(self.mag, other.mag);
        let zero = mag.iter().all(|limb| *limb == 0);
        Self {
            neg: !zero && (self.neg ^ other.neg),
            mag,
        }
    }

    fn to_i128(self) -> i128 {
        if self.is_zero() {
            return 0;
        }
        let fits = self.mag[2] == 0
            && self.mag[3] == 0
            && self.mag[4] == 0
            && self.mag[1] <= i64::MAX as u64;
        if !fits {
            return if self.neg { i128::MIN } else { i128::MAX };
        }
        let v = (self.mag[0] as u128) | ((self.mag[1] as u128) << 64);
        if self.neg {
            -(v as i128)
        } else {
            v as i128
        }
    }
}

fn det(ax: i128, ay: i128, bx: i128, by: i128) -> Wide {
    Wide::from_i128(ax)
        .mul(Wide::from_i128(by))
        .add(Wide::from_i128(bx).mul(Wide::from_i128(ay)).neg())
}

fn lift(x: i128, y: i128) -> Wide {
    let x = Wide::from_i128(x);
    let y = Wide::from_i128(y);
    x.mul(x).add(y.mul(y))
}

impl Wide {
    fn neg(self) -> Self {
        Self {
            neg: !self.is_zero() && !self.neg,
            mag: self.mag,
        }
    }
}

fn add_mag(a: [u64; 5], b: [u64; 5]) -> [u64; 5] {
    let mut out = [0u64; 5];
    let mut carry = 0u128;
    for i in 0..5 {
        carry += a[i] as u128 + b[i] as u128;
        out[i] = carry as u64;
        carry >>= 64;
    }
    debug_assert!(carry == 0, "incircle sum exceeded 320 bits");
    out
}

fn sub_mag(a: [u64; 5], b: [u64; 5]) -> [u64; 5] {
    let mut out = [0u64; 5];
    let mut borrow = 0i128;
    for i in 0..5 {
        let diff = a[i] as i128 - b[i] as i128 - borrow;
        if diff < 0 {
            out[i] = (diff + (1i128 << 64)) as u64;
            borrow = 1;
        } else {
            out[i] = diff as u64;
            borrow = 0;
        }
    }
    debug_assert!(borrow == 0, "magnitude subtract went negative");
    out
}

fn cmp_mag(a: [u64; 5], b: [u64; 5]) -> core::cmp::Ordering {
    for i in (0..5).rev() {
        if a[i] != b[i] {
            return a[i].cmp(&b[i]);
        }
    }
    core::cmp::Ordering::Equal
}

fn mul_mag(a: [u64; 5], b: [u64; 5]) -> [u64; 5] {
    let mut out = [0u64; 5];
    for (i, &ai) in a.iter().enumerate() {
        let mut carry = 0u128;
        for (j, &bj) in b.iter().enumerate() {
            let k = i + j;
            if k >= 5 {
                debug_assert!(carry == 0 && (ai == 0 || bj == 0));
                break;
            }
            let prod = out[k] as u128 + ai as u128 * bj as u128 + carry;
            out[k] = prod as u64;
            carry = prod >> 64;
        }
        debug_assert!(carry == 0, "incircle product exceeded 320 bits");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::incircle;
    use crate::Point;

    #[test]
    fn small_determinant_matches_the_lifted_formula() {
        let a = Point::new(0, 0);
        let b = Point::new(4, 0);
        let c = Point::new(0, 4);
        assert_eq!(incircle(a, b, c, Point::new(1, 1)), 96);
        assert_eq!(incircle(a, b, c, Point::new(2, 2)), 128);
        assert_eq!(incircle(a, b, c, b), 0);
    }

    #[test]
    fn circumcenter_of_a_large_right_triangle_is_inside() {
        let a = Point::new(0, 0);
        let b = Point::new(1 << 32, 0);
        let c = Point::new(0, 1 << 32);
        let center = Point::new(1 << 31, 1 << 31);
        assert!(incircle(a, b, c, center) > 0);
        assert!(incircle(a, b, c, Point::new(1 << 33, 1 << 33)) < 0);
        assert_eq!(incircle(a, b, c, b), 0);
    }
}
