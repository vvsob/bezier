use cgmath::InnerSpace;

use super::vec2;
use super::PolyLine;
use super::Vector2;

use crate::{vertex::RenderData, Vertex};


pub struct ConnectionRenderer {

}

impl ConnectionRenderer {
    pub fn new() -> ConnectionRenderer {
        ConnectionRenderer {}
    }

    pub fn render(&self, line: &PolyLine, width: f64) -> RenderData {
        let mut result = RenderData::new();

        for i in 1..line.points.len() {
            result = result.merge(Self::get_segment_render_data(line, i, width));
        }

        for i in 1..line.points.len() - 1 {
            result = result.merge(Self::get_connection_render_data(line, i, width));
        }

        result
    }

    fn get_segment_render_data(line: &PolyLine, i: usize, width: f64) -> RenderData {
        let start_points = Self::get_adjusted_start_points(line, i - 1, width);
        let end_points = Self::get_adjusted_end_points(line, i, width);
        let vertices: Vec<Vertex> = [start_points.0, start_points.1, end_points.0, end_points.1]
            .map(Vector2::into)
            .map(Vertex::new_f64)
            .into_iter()
            .collect();
        let indices: Vec<_> = vec![0, 2, 3, 0, 3, 1];
        RenderData { vertices, indices }
    }

    fn get_connection_render_data(line: &PolyLine, i: usize, width: f64) -> RenderData {
        let vertices: Vec<_> = match Self::get_connection(line, i, width) {
            Some((intersection, false)) => {
                vec![
                    Self::get_end_points(line, i, width).1,
                    intersection,
                    Self::get_start_points(line, i, width).1,
                ]
            }
            Some((intersection, true)) => {
                vec![
                    Self::get_end_points(line, i, width).0,
                    intersection,
                    Self::get_start_points(line, i, width).0,
                ]
            }
            None => vec![],
        }
        .into_iter()
        .map(Vector2::into)
        .map(Vertex::new_f64)
        .collect();
        let indices = if vertices.is_empty() {
            vec![]
        } else {
            vec![0, 1, 2]
        };
        RenderData { vertices, indices }
    }

    fn get_adjusted_start_points(line: &PolyLine, i: usize, width: f64) -> (Vector2, Vector2) {
        let start_points = Self::get_start_points(line, i, width);
        if i == 0 {
            return start_points;
        }
        match Self::get_connection(line, i, width) {
            Some((intersection, false)) => (intersection, start_points.1),
            Some((intersection, true)) => (start_points.0, intersection),
            None => start_points,
        }
    }

    fn get_adjusted_end_points(line: &PolyLine, i: usize, width: f64) -> (Vector2, Vector2) {
        let end_points = Self::get_end_points(line, i, width);
        if i + 1 == line.points.len() {
            return end_points;
        }
        match Self::get_connection(line, i, width) {
            Some((intersection, false)) => (intersection, end_points.1),
            Some((intersection, true)) => (end_points.0, intersection),
            None => end_points,
        }
    }

    fn get_connection(line: &PolyLine, i: usize, width: f64) -> Option<(Vector2, bool)> {
        let start_points = Self::get_start_points(line, i - 1, width);
        let end_points = Self::get_end_points(line, i, width);
        let next_start_points = Self::get_start_points(line, i, width);
        let next_end_points = Self::get_end_points(line, i + 1, width);
        let lines = (
            make_line(start_points.0, end_points.0),
            make_line(start_points.1, end_points.1),
        );
        let next_lines = (
            make_line(next_start_points.0, next_end_points.0),
            make_line(next_start_points.1, next_end_points.1),
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

    fn get_start_points(line: &PolyLine, i: usize, width: f64) -> (Vector2, Vector2) {
        if i + 1 == line.points.len() {
            panic!();
        }
        Self::offset_by_direction(
            line.points[i],
            (line.points[i + 1] - line.points[i]).normalize() * width,
        )
    }

    fn get_end_points(line: &PolyLine, i: usize, width: f64) -> (Vector2, Vector2) {
        if i == 0 {
            panic!();
        }
        Self::offset_by_direction(
            line.points[i],
            (line.points[i] - line.points[i - 1]).normalize() * width,
        )
    }

    fn offset_by_direction(point: Vector2, direction: Vector2) -> (Vector2, Vector2) {
        (
            point + vec2(direction.y, -direction.x),
            point + vec2(-direction.y, direction.x),
        )
    }
}

fn make_line(start: Vector2, end: Vector2) -> geo::Line<f64> {
    geo::Line {
        start: geo::Coord {
            x: start.x,
            y: start.y,
        },
        end: geo::Coord { x: end.x, y: end.y },
    }
}