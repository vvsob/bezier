pub mod renderer;

type Vector2 = cgmath::Vector2<f64>;

fn vec2(x: f64, y: f64) -> Vector2 {
    cgmath::vec2(x, y)
}

pub struct Bezier {
    pub start: Vector2,
    pub middle: Vector2,
    pub end: Vector2,
}

impl Bezier {
    pub fn subdivide(&self, count: usize) -> PolyLine {
        PolyLine {
            points: (0..count)
                .map(|i| self.eval((i as f64) / (count - 1) as f64))
                .collect(),
        }
    }

    pub fn new(start: Vector2, middle: Vector2, end: Vector2) -> Self {
        Self { start, middle, end }
    }

    fn eval(&self, t: f64) -> Vector2 {
        let a = Self::lerp(self.start, self.middle, t);
        let b = Self::lerp(self.middle, self.end, t);
        Self::lerp(a, b, t)
    }

    fn lerp(start: Vector2, end: Vector2, t: f64) -> Vector2 {
        end * t + start * (1.0 - t)
    }
}

pub struct PolyLine {
    pub points: Vec<Vector2>,
}

impl PolyLine {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }
}
