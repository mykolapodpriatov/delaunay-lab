use crate::point::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Distribution {
    Uniform,
    Circle,
    Clusters,
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn unit(&mut self) -> f64 {
        (self.next_u32() as f64) / (u32::MAX as f64)
    }
}

pub fn generate(seed: u64, dist: Distribution, n: usize, scale: i64) -> Vec<Point> {
    let mut rng = Lcg::new(seed);
    match dist {
        Distribution::Uniform => (0..n)
            .map(|_| {
                let x = (rng.unit() * scale as f64) as i64;
                let y = (rng.unit() * scale as f64) as i64;
                Point::new(x, y)
            })
            .collect(),
        Distribution::Circle => (0..n)
            .map(|i| {
                let t = (i as f64) * std::f64::consts::TAU / n.max(1) as f64;
                let r = (scale as f64) * 0.45;
                Point::new(
                    (scale as f64 * 0.5 + r * t.cos()) as i64,
                    (scale as f64 * 0.5 + r * t.sin()) as i64,
                )
            })
            .collect(),
        Distribution::Clusters => {
            let centers = [
                (scale / 4, scale / 4),
                (3 * scale / 4, scale / 4),
                (scale / 2, 3 * scale / 4),
            ];
            (0..n)
                .map(|i| {
                    let (cx, cy) = centers[i % 3];
                    let jx = ((rng.unit() - 0.5) * scale as f64 * 0.16) as i64;
                    let jy = ((rng.unit() - 0.5) * scale as f64 * 0.16) as i64;
                    Point::new(cx + jx, cy + jy)
                })
                .collect()
        }
    }
}
