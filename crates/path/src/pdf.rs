use crate::{
    builder::nan_check,
    math::{point, Point},
    path::Verb,
    traits::Build,
    Attributes, EndpointId, Path,
};
use lyon_geom::LineSegment;
use std::convert::TryInto;

pub struct Pdf {
    pub(crate) points: Vec<Point>,
    pub(crate) verbs: Vec<Verb>,
    first_position: Point,
    need_moveto: bool,
    is_empty: bool,
}

impl Pdf {
    pub fn new() -> Self {
        Pdf {
            points: vec![],
            verbs: vec![],
            // TODO: start at bottom left of page, I think
            first_position: point(0.0, 0.0),
            need_moveto: true,
            is_empty: true,
        }
    }

    pub fn move_to(&mut self, to: Point) -> EndpointId {
        self.end_if_needed();

        let id = self.begin(to, None);

        self.is_empty = false;
        self.need_moveto = false;
        self.first_position = to;

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

    pub fn line_to(&mut self, to: Point) -> EndpointId {
        if let Some(id) = self.begin_if_needed(&to) {
            return id;
        }

        // TODO: Create validator
        // self.validator.edge();
        nan_check(to);

        let id = EndpointId(self.points.len() as u32);
        self.points.push(to);
        self.verbs.push(Verb::LineTo);

        id
    }

    #[inline(always)]
    fn begin_if_needed(&mut self, default: &Point) -> Option<EndpointId> {
        if self.need_moveto {
            return self.insert_move_to(default);
        }

        None
    }

    #[inline(never)]
    fn insert_move_to(&mut self, default: &Point) -> Option<EndpointId> {
        if self.is_empty {
            return Some(self.move_to(*default));
        }

        self.move_to(self.first_position);

        None
    }

    pub fn close(&mut self) {
        if self.need_moveto {
            return;
        }

        self.need_moveto = true;

        self.end(true)
    }

    pub fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) -> EndpointId {
        if let Some(id) = self.begin_if_needed(&to) {
            return id;
        }

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
        if let Some(id) = self.begin_if_needed(&to) {
            return id;
        }

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
    fn make_fillable_if_needed(&mut self) {
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
        self.make_fillable_if_needed();

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
    use lyon_geom::{
        euclid::{Point2D, UnknownUnit},
        LineSegment,
    };

    /// Just a single line
    #[test]
    fn test_as_rect_single_line() {
        let mut pdf = Pdf::new();
        pdf.points.append(&mut vec![
            // moveto
            point(0.0, 792.0),
            // moveto
            point(10.0, 10.0),
            // lineto
            point(10.0, 20.0),
        ]);
        pdf.verbs
            .append(&mut vec![Begin, End, Begin, LineTo, Close]);
        let path = pdf.build();

        let expected_points: Box<[Point2D<f32, UnknownUnit>]> = Box::new([
            // moveto
            point(0.0, 792.0),
            // moveto
            point(9.5, 20.0),
            // lineto
            point(10.5, 20.0),
            // lineto
            point(10.5, 10.0),
            // lineto
            point(9.5, 10.0),
        ]);
        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> =
            Box::new([Begin, End, Begin, LineTo, LineTo, LineTo, Close]);
        assert_eq!(path.verbs, expected_verbs);
    }

    /// Multiple lines
    #[test]
    fn test_as_rect_multiple_lines() {
        let mut pdf = Pdf::new();
        pdf.points.append(&mut vec![
            // moveto
            point(0.0, 792.0),
            // moveto
            point(10.0, 10.0),
            // lineto
            point(10.0, 20.0),
            // moveto
            point(10.0, 10.0),
            // lineto
            point(10.0, 20.0),
            // moveto
            point(0.0, 0.0),
            // lineto
            point(10.0, 20.0),
        ]);
        pdf.verbs.append(&mut vec![
            Begin, End, Begin, LineTo, End, Begin, LineTo, End, Begin, LineTo, Close,
        ]);
        let path = pdf.build();

        let expected_points: Box<[Point]> = Box::new([
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(9.5, 20.0),
            // lineto:
            point(10.5, 20.0),
            point(10.5, 10.0),
            point(9.5, 10.0),
            // moveto:
            point(9.5, 20.0),
            // lineto:
            point(10.5, 20.0),
            point(10.5, 10.0),
            point(9.5, 10.0),
            // moveto
            point(9.552786, 20.223608),
            // lineto
            point(10.447212, 19.776394),
            point(0.4472146, -0.22360802),
            point(-0.4472127, 0.22360611),
        ]);
        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([
            Begin, End, Begin, LineTo, LineTo, LineTo, End, Begin, LineTo, LineTo, LineTo, End,
            Begin, LineTo, LineTo, LineTo, Close,
        ]);
        assert_eq!(path.verbs, expected_verbs);
    }

    /// A line followed by a cubic bezier
    #[test]
    fn test_as_rect_line_then_cubic_bez() {
        let mut pdf = Pdf::new();
        pdf.points.append(&mut vec![
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(10.0, 10.0),
            //lineto:
            point(10.0, 20.0),
            // moveto:
            point(20.0, 20.0),
            // cubic bez to:
            point(25.0, 25.0),
            point(35.0, 35.0),
            point(40.0, 25.0),
        ]);
        pdf.verbs.append(&mut vec![
            Begin, End, Begin, LineTo, End, Begin, CubicTo, Close,
        ]);
        let path = pdf.build();

        let expected_points: Box<[Point]> = Box::new([
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(9.5, 20.0),
            //lineto:
            point(10.5, 20.0),
            point(10.5, 10.0),
            point(9.5, 10.0),
            // moveto:
            point(20.0, 20.0),
            // cubic bez to:
            point(25.0, 25.0),
            point(35.0, 35.0),
            point(40.0, 25.0),
        ]);
        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([
            Begin, End, Begin, LineTo, LineTo, LineTo, End, Begin, CubicTo, Close,
        ]);
        assert_eq!(path.verbs, expected_verbs);
    }

    /// A cubic bezier followed by a line
    #[test]
    fn test_as_rect_cubic_bez_then_line() {
        let mut pdf = Pdf::new();
        pdf.points.append(&mut vec![
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(20.0, 20.0),
            // cubic bez to:
            point(25.0, 25.0),
            point(35.0, 35.0),
            point(40.0, 25.0),
            // moveto:
            point(10.0, 10.0),
            //lineto:
            point(10.0, 20.0),
        ]);
        pdf.verbs.append(&mut vec![
            Begin, End, Begin, CubicTo, Close, Begin, LineTo, End,
        ]);
        let path = pdf.build();

        let expected_points: Box<[Point]> = Box::new([
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(20.0, 20.0),
            // cubic bez to:
            point(25.0, 25.0),
            point(35.0, 35.0),
            point(40.0, 25.0),
            // moveto:
            point(9.5, 20.0),
            //lineto:
            point(10.5, 20.0),
            point(10.5, 10.0),
            point(9.5, 10.0),
        ]);

        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([
            Begin, End, Begin, CubicTo, Close, Begin, LineTo, LineTo, LineTo, End,
        ]);
        assert_eq!(path.verbs, expected_verbs);
    }

    /// A couple of movetos followed by a line
    #[test]
    fn test_as_rect_moveto_then_line() {
        let mut pdf = Pdf::new();
        pdf.points.append(&mut vec![
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(20.0, 20.0),
            // moveto:
            point(21.0, 21.0),
            // cubic bez to:
            point(25.0, 25.0),
            point(35.0, 35.0),
            point(40.0, 25.0),
            // moveto:
            point(10.0, 10.0),
            // moveto:
            point(11.0, 11.0),
            //lineto:
            point(10.0, 20.0),
        ]);
        pdf.verbs.append(&mut vec![
            Begin, End, Begin, End, Begin, CubicTo, Close, Begin, End, Begin, LineTo, End,
        ]);
        let path = pdf.build();

        let expected_points: Box<[Point]> = Box::new([
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(20.0, 20.0),
            // moveto:
            point(21.0, 21.0),
            // cubic bez to:
            point(25.0, 25.0),
            point(35.0, 35.0),
            point(40.0, 25.0),
            // moveto:
            point(10.0, 10.0),
            // moveto:
            point(9.5030575, 19.944784),
            //lineto:
            point(10.496942, 20.055216),
            point(11.4969425, 11.055216),
            point(10.503058, 10.944784),
        ]);

        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([
            Begin, End, Begin, End, Begin, CubicTo, Close, Begin, End, Begin, LineTo, LineTo,
            LineTo, End,
        ]);
        assert_eq!(path.verbs, expected_verbs);
    }

    /// A line followed by a cubic bezier
    #[test]
    fn test_as_rect_line_then_quadratic_bez() {
        let mut pdf = Pdf::new();
        pdf.points.append(&mut vec![
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(20.0, 20.0),
            // lineto:
            point(10.0, 20.0),
            // moveto:
            point(20.0, 20.0),
            // quad bez:
            point(25.0, 25.0),
            point(20.0, 20.0),
        ]);
        pdf.verbs.append(&mut vec![
            Begin,
            End,
            Begin,
            LineTo,
            Close,
            Begin,
            QuadraticTo,
            End,
        ]);
        let path = pdf.build();

        let expected_points: Box<[Point]> = Box::new([
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(10.0, 19.5),
            // lineto:
            point(10.0, 20.5),
            point(20.0, 20.5),
            point(20.0, 19.5),
            // moveto:
            point(20.0, 20.0),
            // quad bez:
            point(25.0, 25.0),
            point(20.0, 20.0),
        ]);

        assert_eq!(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([
            Begin,
            End,
            Begin,
            LineTo,
            LineTo,
            LineTo,
            Close,
            Begin,
            QuadraticTo,
            End,
        ]);
        assert_eq!(path.verbs, expected_verbs);
    }

    /// A cubic bezier followed by a line
    #[test]
    fn test_as_rect_quadratic_bez_then_line() {
        let mut pdf = Pdf::new();
        pdf.points.append(&mut vec![
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(20.0, 20.0),
            // quad bez:
            point(25.0, 25.0),
            point(20.0, 20.0),
            // moveto:
            point(20.0, 20.0),
            // lineto:
            point(10.0, 20.0),
        ]);
        pdf.verbs.append(&mut vec![
            Begin,
            End,
            Begin,
            QuadraticTo,
            End,
            Begin,
            LineTo,
            Close,
        ]);
        let path = pdf.build();

        let expected_points: Box<[Point]> = Box::new([
            // moveto:
            point(0.0, 792.0),
            // moveto:
            point(20.0, 20.0),
            // quad bez:
            point(25.0, 25.0),
            point(20.0, 20.0),
            // moveto:
            point(10.0, 19.5),
            // lineto:
            point(10.0, 20.5),
            point(20.0, 20.5),
            point(20.0, 19.5),
        ]);

        assert_relative_eq_boxed_pt_slice(path.points, expected_points);
        let expected_verbs: Box<[Verb]> = Box::new([
            Begin,
            End,
            Begin,
            QuadraticTo,
            End,
            Begin,
            LineTo,
            LineTo,
            LineTo,
            Close,
        ]);
        assert_eq!(path.verbs, expected_verbs);
    }
}
