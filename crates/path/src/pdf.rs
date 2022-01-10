use crate::{
    builder::nan_check,
    math::{point, Point, Vector},
    path::Verb,
    traits::Build,
    Attributes, EndpointId, Path,
};
use lyon_geom::{vector, LineSegment};
use shared::{PageHeight, PageWidth};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Pdf {
    pub(crate) points: Vec<Point>,
    pub(crate) verbs: Vec<Verb>,
    first_position: Point,
    current_position: Point,
    page_width: PageWidth,
    page_height: PageHeight,
}

impl Pdf {
    pub fn new(page_width: PageWidth, page_height: PageHeight) -> Self {
        let mut p = Pdf {
            points: vec![],
            verbs: vec![],
            first_position: point(-page_width / 2.0, page_height / 2.0),
            current_position: point(-page_width / 2.0, page_height / 2.0),
            page_width,
            page_height,
        };
        p.move_to_abs(p.first_position);
        p
    }

    pub fn move_to(&mut self, to: Vector) -> EndpointId {
        self.end_if_needed();

        let to = vector(to.x, -to.y);
        let to = self.first_position + to;
        let id = self.begin(to, None);

        self.current_position = to;

        id
    }

    fn move_to_abs(&mut self, to: Point) -> EndpointId {
        self.end_if_needed();

        let id = self.begin(to, None);

        self.current_position = to;

        id
    }

    fn begin(&mut self, at: Point, _attributes: Option<Attributes>) -> EndpointId {
        // TODO: Add validator
        // self.validator.begin();
        nan_check(at);

        let id = EndpointId(self.points.len() as u32);
        self.points.push(at);
        self.verbs.push(Verb::Begin);

        id
    }

    fn end_if_needed(&mut self) {
        let maybe_last = self.verbs.last();
        if maybe_last.is_some() && (*maybe_last.unwrap() as u8) <= (Verb::Begin as u8) {
            self.end(false);
        }
    }

    fn end(&mut self, close: bool) {
        // TODO: Add validator
        // self.validator.end();

        self.verbs.push(if close { Verb::Close } else { Verb::End });
    }

    pub fn line_to(&mut self, to: Vector) -> EndpointId {
        // TODO: assert that there is a moveto command in the subpath? Not
        // sure this is needed for PDFs but it's in the WithSVG impl

        let to = vector(to.x, -to.y);
        let to = self.first_position + to;

        // TODO: Create validator
        // self.validator.edge();
        nan_check(to);

        let id = EndpointId(self.points.len() as u32);
        self.points.push(to);
        self.verbs.push(Verb::LineTo);

        id
    }

    pub fn close(&mut self) {
        // TODO: Assert some stuff about path validity
        self.end(true)
    }

    pub fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) -> EndpointId {
        // TODO: assert that there is a moveto command in the subpath? Not
        // sure this is needed for PDFs but it's in the WithSVG impl

        // TODO: Not sure whether any of this is needed for any reason. Seems unlikely
        // self.current_position = to;
        // self.last_cmd = Verb::CubicTo;
        // self.last_ctrl = ctrl2;

        // TODO: Add validator
        // self.validator.edge();
        nan_check(ctrl1);
        nan_check(ctrl2);
        nan_check(to);

        self.points.push(ctrl1);
        self.points.push(ctrl2);
        let id = EndpointId(self.points.len() as u32);
        self.points.push(to);
        self.verbs.push(Verb::CubicTo);

        id
    }

    pub fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point) -> EndpointId {
        // TODO: assert that there is a moveto command in the subpath? Not
        // sure this is needed for PDFs but it's in the WithSVG impl

        // TODO: Add validator
        // self.validator.edge();
        nan_check(ctrl);
        nan_check(to);

        self.points.push(ctrl);
        let id = EndpointId(self.points.len() as u32);
        self.points.push(to);
        self.verbs.push(Verb::QuadraticTo);

        id
    }

    /// For any single linetos ending with a fill, makes them rectangles that can be filled
    pub fn make_fillable_if_needed(&mut self) {
        let mut points = self.points.iter().enumerate();
        let mut lineto_insertions = vec![];
        let mut point_replacements = vec![];
        let mut point_insertions = vec![];
        let mut skip_next_n_windows = 0;
        for (first_item_index, window) in self.verbs.windows(3).enumerate() {
            if skip_next_n_windows > 0 {
                skip_next_n_windows -= 1;
                continue;
            }
            let [a, b, c]: [Verb; 3] = window.try_into().unwrap();
            let maybe_from = match a {
                Verb::Close | Verb::End => None,
                Verb::CubicTo => {
                    // there are two `control` points and one `to` point in a cubic curve
                    points.next();
                    points.next();
                    points.next();
                    None
                }
                Verb::Begin | Verb::LineTo => points.next(),
                Verb::QuadraticTo => {
                    // there is one `control` points and one `to` point in a cubic curve
                    points.next();
                    points.next();
                    None
                }
            };
            if a == Verb::Begin && b == Verb::LineTo && (c == Verb::Close || c == Verb::End) {
                // The next couple of .windows(3) calls will be `LineTo, Close/End, ???` and
                // `Close/End, ???, ???`. We can skip these two, and we must do so in order
                // for our point iterator to work properly and not get ahead of itself
                skip_next_n_windows += 2;
                lineto_insertions.push(first_item_index + 2);
                lineto_insertions.push(first_item_index + 2);
                let (i, from) = maybe_from.unwrap();
                let (j, to) = points.next().unwrap();
                let line = LineSegment {
                    from: *from,
                    to: *to,
                };
                let rect_points = line.as_rect();
                point_replacements.push((i, rect_points[0]));
                point_replacements.push((j, rect_points[1]));
                // Have to insert in this order so we don't get an `out of range` error below
                point_insertions.push((j + 2, rect_points[3]));
                point_insertions.push((j + 1, rect_points[2]));
            }
        }

        for i in lineto_insertions.into_iter().rev() {
            self.verbs.insert(i, Verb::LineTo);
        }

        for (i, point) in point_replacements.into_iter() {
            self.points[i] = point;
        }

        for (i, point) in point_insertions.into_iter().rev() {
            self.points.insert(i, point);
        }
    }
}

impl Build for Pdf {
    type PathType = Path;

    fn build(mut self) -> Path {
        self.end_if_needed();
        // TODO: Implement validator
        // self.validator.build();

        Path {
            points: self.points.into_boxed_slice(),
            verbs: self.verbs.into_boxed_slice(),
            num_attributes: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{path::Verb::*, test_utils::assert_relative_eq_boxed_pt_slice};
    use lyon_geom::euclid::{Point2D, UnknownUnit};

    #[test]
    fn test_empty_path() {
        let w = PageWidth::new(800.0);
        let h = PageHeight::new(800.0);
        let mut pdf = Pdf::new(w, h);
        let path = pdf.build();

        let expected_points: Box<[Point2D<f32, UnknownUnit>]> = Box::new([
            // MoveTo:
            point(-400.0, 400.0),
        ]);
        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([Begin, End]);
        assert_eq!(path.verbs, expected_verbs);
    }

    #[test]
    fn test_converts_single_line_to_rect() {
        let w = PageWidth::new(800.0);
        let h = PageHeight::new(800.0);
        let mut pdf = Pdf::new(w, h);
        pdf.line_to(vector(10.0, 10.0));
        pdf.close();
        pdf.make_fillable_if_needed();
        let path = pdf.build();

        let expected_points: Box<[Point2D<f32, UnknownUnit>]> = Box::new([
            // MoveTo:
            point(-389.64645, 390.35355),
            // LineTo:
            point(-390.35355, 389.64645),
            point(-400.35355, 399.64645),
            point(-399.64645, 400.35355),
        ]);
        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([Begin, LineTo, LineTo, LineTo, Close]);
        assert_eq!(path.verbs, expected_verbs);
    }
}
