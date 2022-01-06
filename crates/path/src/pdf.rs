use crate::{
    builder::nan_check,
    math::{point, Point},
    path::Verb,
    traits::Build,
    Attributes, EndpointId, Path,
};
use lyon_geom::{LineSegment, euclid::{Point2D, UnknownUnit}};
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

    /// For any single lines ending with a fill, makes them rectangles that can be filled
    fn make_fillable_if_needed(&mut self) {
        // TODO: Need to make this dynamic to work with any number of lines
        // self.verbs.insert(4, Verb::LineTo);
        // self.verbs.insert(5, Verb::LineTo);

        // begin, lineto, close
        // begin, lineto, end, begin, lineto, end, close (need to verify that this is a valid arrangement)

        // begin, lineto, end or close
        let mut points = self.points.iter().enumerate();
        let mut lineto_insertions = vec![];
        let mut point_replacements = vec![];
        let mut point_insertions = vec![];
        // [(0.0, 792.0), (10.0, 10.0), (10.0, 20.0)]
        // [Begin, End, Begin, LineTo, Close]
        for (first_item_index, window) in self.verbs.windows(3).enumerate() {
            let [a, b, c]: [Verb; 3] = window.try_into().unwrap();
            // if a is not Close/End, move the points iter forward one
            let maybe_from = if a != Verb::Close && a != Verb::End {
                points.next()
            } else {
                None
            };
            if a == Verb::Begin && b == Verb::LineTo && (c == Verb::Close || c == Verb::End) {
                lineto_insertions.push(first_item_index + 2);
                lineto_insertions.push(first_item_index + 2);
                let (i, from) = maybe_from.unwrap();
                let (j, to) = points.next().unwrap();
                let line = LineSegment { from: *from, to: *to };
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
        // panic!("{:?}", self.points);

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

// TODO: Make real
// #[test]
// fn simple_path() {
//     let mut builder = Pdf::new();
//     builder.move_to(point(100.0, 100.0));
//     builder.close();
//     let path = builder.build();

//     panic!("{:?}", path);
// }

#[test]
fn test_as_rect() {
    let mut pdf = Pdf::new();
    pdf.points.append(&mut vec![
        point(0.0, 792.0),
        point(10.0, 10.0),
        point(10.0, 20.0),
    ]);
    pdf.verbs.append(&mut vec![
        Verb::Begin,
        Verb::End,
        Verb::Begin,
        Verb::LineTo,
        Verb::Close,
    ]);
    let path = pdf.build();

    let expected_points: Box<[Point2D<f32, UnknownUnit>]> = Box::new([
        point(0.0, 792.0),
        point(9.5, 20.0),
        point(10.5, 20.0),
        point(10.5, 10.0),
        point(9.5, 10.0),
    ]);
    assert_eq!(path.points, expected_points);
    let expected_verbs: Box<[Verb]> = Box::new([
        Verb::Begin,
        Verb::End,
        Verb::Begin,
        Verb::LineTo,
        Verb::LineTo,
        Verb::LineTo,
        Verb::Close,
    ]);
    assert_eq!(path.verbs, expected_verbs);
}
