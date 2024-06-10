use cgmath::InnerSpace;

use crate::{vertex::RenderData, Vertex};

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

    pub fn get_render_data(&self, width: f64) -> RenderData {
        let mut result = RenderData::new();

        for i in 1..self.points.len() {
            result = result.merge(self.get_segment_render_data(i, width));
        }

        for i in 1..self.points.len() - 1 {
            result = result.merge(self.get_connection_render_data(i, width));
        }

        result
    }

    fn get_segment_render_data(&self, i: usize, width: f64) -> RenderData {
        let start_points = self.get_adjusted_start_points(i - 1, width);
        let end_points = self.get_adjusted_end_points(i, width);
        let vertices: Vec<Vertex> = [start_points.0, start_points.1, end_points.0, end_points.1]
            .map(Vector2::into)
            .map(Vertex::new_f64)
            .into_iter()
            .collect();
        let indices: Vec<_> = vec![0, 2, 3, 0, 3, 1];
        RenderData { vertices, indices }
    }

    fn get_connection_render_data(&self, i: usize, width: f64) -> RenderData {
        let vertices: Vec<_> = match self.get_connection(i, width) {
            Some((intersection, false)) => {
                vec![
                    self.get_end_points(i, width).1,
                    intersection,
                    self.get_start_points(i, width).1,
                ]
            }
            Some((intersection, true)) => {
                vec![
                    self.get_end_points(i, width).0,
                    intersection,
                    self.get_start_points(i, width).0,
                ]
            }
            None => vec![],
        }
        .into_iter()
        .map(Vector2::into)
        .map(Vertex::new_f64)
        .into_iter()
        .collect();
        let indices = if vertices.is_empty() {
            vec![]
        } else {
            vec![0, 1, 2]
        };
        RenderData { vertices, indices }
    }

    fn get_adjusted_start_points(&self, i: usize, width: f64) -> (Vector2, Vector2) {
        let start_points = self.get_start_points(i, width);
        if i == 0 {
            return start_points;
        }
        match self.get_connection(i, width) {
            Some((intersection, false)) => (intersection, start_points.1),
            Some((intersection, true)) => (start_points.0, intersection),
            None => start_points,
        }
    }

    fn get_adjusted_end_points(&self, i: usize, width: f64) -> (Vector2, Vector2) {
        let end_points = self.get_end_points(i, width);
        if i + 1 == self.points.len() {
            return end_points;
        }
        match self.get_connection(i, width) {
            Some((intersection, false)) => (intersection, end_points.1),
            Some((intersection, true)) => (end_points.0, intersection),
            None => end_points,
        }
    }

    fn get_connection(&self, i: usize, width: f64) -> Option<(Vector2, bool)> {
        let start_points = self.get_start_points(i - 1, width);
        let end_points = self.get_end_points(i, width);
        let next_start_points = self.get_start_points(i, width);
        let next_end_points = self.get_end_points(i + 1, width);
        let lines = (
            Self::line(start_points.0, end_points.0),
            Self::line(start_points.1, end_points.1),
        );
        let next_lines = (
            Self::line(next_start_points.0, next_end_points.0),
            Self::line(next_start_points.1, next_end_points.1),
        );

        use geo::algorithm::line_intersection::{line_intersection, LineIntersection};

        let intersections = (
            line_intersection(lines.0, next_lines.0),
            line_intersection(lines.1, next_lines.1),
        );

        match intersections {
            (Some(LineIntersection::SinglePoint { intersection, .. }), None) => {
                Some((Vector2::new(intersection.x, intersection.y), false))
            }
            (None, Some(LineIntersection::SinglePoint { intersection, .. })) => {
                Some((Vector2::new(intersection.x, intersection.y), true))
            }
            _ => None,
        }
    }

    fn get_start_points(&self, i: usize, width: f64) -> (Vector2, Vector2) {
        if i + 1 == self.points.len() {
            panic!();
        }
        Self::offset_by_direction(
            self.points[i],
            (self.points[i + 1] - self.points[i]).normalize() * width,
        )
    }

    fn get_end_points(&self, i: usize, width: f64) -> (Vector2, Vector2) {
        if i == 0 {
            panic!();
        }
        Self::offset_by_direction(
            self.points[i],
            (self.points[i] - self.points[i - 1]).normalize() * width,
        )
    }

    fn line(start: Vector2, end: Vector2) -> geo::Line<f64> {
        geo::Line {
            start: geo::Coord {
                x: start.x,
                y: start.y,
            },
            end: geo::Coord { x: end.x, y: end.y },
        }
    }

    fn offset_by_direction(point: Vector2, direction: Vector2) -> (Vector2, Vector2) {
        (
            point + vec2(direction.y, -direction.x),
            point + vec2(-direction.y, direction.x),
        )
    }
}
