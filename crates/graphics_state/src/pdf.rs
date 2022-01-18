// use crate::{
//     builder::nan_check,
//     math::{point, vector, Point, Vector},
//     path::Verb,
//     traits::Build,
//     Attributes, EndpointId, Path,
// };
use lyon::{geom::{LineSegment, vector, Scalar, Rotation, Translation, Angle}, path::{EndpointId, Attributes, Path, traits::Build}, geom::{Point as SPoint, point as s_point}, math::{Point, point, Vector}};
use shared::{Height, Width};
use std::convert::TryInto;

/// Enumeration corresponding to the [Event](https://docs.rs/lyon_core/*/lyon_core/events/enum.Event.html) enum
/// without the parameters.
///
/// This is used by the [Path](struct.Path.html) data structure to store path events a tad
/// more efficiently.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub(crate) enum Verb {
    LineTo,
    QuadraticTo,
    CubicTo,
    Begin,
    Close,
    End,
}

#[inline]
pub fn nan_check(p: Point) {
    debug_assert!(p.x.is_finite());
    debug_assert!(p.y.is_finite());
}

#[inline]
pub fn get_pythagorean_hypotenuse<S: Scalar>(p1: SPoint<S>, p2: SPoint<S>) -> S {
    let a_squared = (p1.y - p2.y).abs().powi(2);
    let b_squared = (p1.x - p2.x).abs().powi(2);
    let c_squared = a_squared + b_squared;
    c_squared.sqrt()
}

pub fn as_rect<S: Scalar>(segment: LineSegment<S>) -> [SPoint<S>; 4] {
    let from = segment.from;
    let to = segment.to;
    let fill_width = S::ONE;
    let hypotenuse = get_pythagorean_hypotenuse(from, to);
    let p1 = s_point(-fill_width / S::TWO, hypotenuse / S::TWO);
    let p2 = s_point(fill_width / S::TWO, hypotenuse / S::TWO);
    let p3 = s_point(fill_width / S::TWO, -hypotenuse / S::TWO);
    let p4 = s_point(-fill_width / S::TWO, -hypotenuse / S::TWO);

    let x_mid = (from.x + to.x) / S::TWO;
    let y_mid = (from.y + to.y) / S::TWO;
    let degrees =
        S::atan2(to.y - from.y, to.x - from.x).to_degrees() - (S::TEN * S::NINE);
    let rotation = Rotation::new(Angle::degrees(degrees));
    let p1 = rotation.transform_point(p1);
    let p2 = rotation.transform_point(p2);
    let p3 = rotation.transform_point(p3);
    let p4 = rotation.transform_point(p4);

    let translation = Translation::new(x_mid, y_mid);
    let p1 = translation.transform_point(p1);
    let p2 = translation.transform_point(p2);
    let p3 = translation.transform_point(p3);
    let p4 = translation.transform_point(p4);

    [p1, p2, p3, p4]
}


#[derive(Debug)]
pub struct Pdf {
    pub(crate) points: Vec<Point>,
    pub(crate) verbs: Vec<Verb>,
    current_position: Point,
    bottom_left_corner: Point,
    need_moveto: bool,
    is_empty: bool,
    page_width: Width,
    page_height: Height,
}

impl Pdf {
    pub fn new(page_width: Width, page_height: Height) -> Self {
        Pdf {
            points: vec![],
            verbs: vec![],
            current_position: point(0.0, 0.0),
            bottom_left_corner: point(-page_width / 2.0, page_height / 2.0),
            need_moveto: true,
            is_empty: true,
            page_width,
            page_height,
        }
    }

    fn move_to_abs(&mut self, to: Point) -> EndpointId {
        self.end_if_needed();

        let id = self.begin(to, None);

        self.current_position = to;
        self.is_empty = false;
        self.need_moveto = false;
        id
    }

    pub fn move_to(&mut self, to: Vector) -> EndpointId {
        self.end_if_needed();

        let to = vector(to.x, -to.y);
        let to = self.bottom_left_corner + to;
        let id = self.begin(to, None);

        self.current_position = to;
        self.is_empty = false;
        self.need_moveto = false;

        id
    }

    fn begin(&mut self, at: Point, _attributes: Option<Attributes>) -> EndpointId {
        // TODO: Add validator
        // self.validator.begin();
        // nan_check(at);

        let id = EndpointId(self.points.len() as u32);
        self.points.push(at);
        self.verbs.push(Verb::Begin);

        id
    }

    /// Ensures the current sub-path has a moveto command.
    ///
    /// Returns an ID if the command should be skipped and the ID returned instead.
    #[inline(always)]
    fn begin_if_needed(&mut self, default: &Vector) -> Option<EndpointId> {
        if self.need_moveto {
            return self.insert_move_to(default);
        }

        None
    }

    #[inline(never)]
    fn insert_move_to(&mut self, _default: &Vector) -> Option<EndpointId> {
        // if nothing in path, go to bottom corner of page
        if self.is_empty {
            return Some(self.move_to_abs(point(-self.page_width / 2.0, self.page_height / 2.0)));
        }
        // TODO: Not sure about this. Test a scenario that would trip it up. E.g.
        // a LineTo without a MoveTo in front of it, but with is_empty == false.
        // E.g.
        // 10 10 m
        // 10 10 l
        // 20 20 l
        self.move_to_abs(self.current_position);

        None
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
        self.begin_if_needed(&to);

        // TODO: assert that there is a moveto command in the subpath? Not
        // sure this is needed for PDFs but it's in the WithSVG impl

        let to = vector(to.x, -to.y);
        let to = self.bottom_left_corner + to;

        // TODO: Create validator
        // self.validator.edge();
        nan_check(to);

        let id = EndpointId(self.points.len() as u32);
        self.points.push(to);
        self.verbs.push(Verb::LineTo);

        id
    }

    pub fn rect(&mut self, low_left: Vector, width: Width, height: Height) {
        let width = f32::max(*width, 1.0);
        let height = f32::max(*height, 1.0);
        self.move_to(low_left);
        let to = vector(low_left.x + width, low_left.y);
        self.line_to(to);
        let to = vector(low_left.x + width, low_left.y + height);
        self.line_to(to);
        let to = vector(low_left.x, low_left.y + height);
        self.line_to(to);
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
                let (i, from) = maybe_from.unwrap();
                let (j, to) = points.next().unwrap();
                let from = *from;
                let to = *to;
                // Ensure that the line will be visible even if it's only a dot:
                let absolute_change = (self.current_position - to).abs();
                let minimum_distance = vector(0.6, 0.6);
                let to = if absolute_change.x < minimum_distance.x
                    && absolute_change.y < minimum_distance.y
                {
                    point(self.current_position.x + minimum_distance.x, to.y)
                } else {
                    to
                };
                skip_next_n_windows += 2;
                lineto_insertions.push(first_item_index + 2);
                lineto_insertions.push(first_item_index + 2);
                let line = LineSegment { from, to };
                let rect_points = as_rect(line);
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

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::{test_utils::assert_relative_eq_boxed_pt_slice};
//     use approx::assert_relative_eq;
//     use lyon::lyon_extra::euclid::{Point2D, UnknownUnit};

//     #[test]
//     fn test_empty_path() {
//         let w = Width::new(800.0);
//         let h = Height::new(800.0);
//         let pdf = Pdf::new(w, h);
//         let path = pdf.build();

//         let expected_points: Box<[Point2D<f32, UnknownUnit>]> = Box::new([]);
//         assert_relative_eq_boxed_pt_slice(path.points, expected_points);
//         let expected_verbs: Box<[Verb]> = Box::new([]);
//         assert_eq!(path.verbs, expected_verbs);
//     }

//     #[test]
//     fn test_converts_single_line_to_rect() {
//         let w = Width::new(800.0);
//         let h = Height::new(800.0);
//         let mut pdf = Pdf::new(w, h);
//         pdf.line_to(vector(10.0, 10.0));
//         pdf.close();
//         pdf.make_fillable_if_needed();
//         let path = pdf.build();

//         let expected_points: Box<[Point2D<f32, UnknownUnit>]> = Box::new([
//             // MoveTo:
//             point(-389.64645, 390.35355),
//             // LineTo:
//             point(-390.35355, 389.64645),
//             point(-400.35355, 399.64645),
//             point(-399.64645, 400.35355),
//         ]);
//         assert_relative_eq_boxed_pt_slice(path.points, expected_points);
//         let expected_verbs: Box<[Verb]> = Box::new([Begin, LineTo, LineTo, LineTo, Close]);
//         assert_eq!(path.verbs, expected_verbs);
//     }
// }
