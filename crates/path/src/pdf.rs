use lyon_geom::LineSegment;

use crate::{math::{Point, point}, path::Verb, EndpointId, Path, Attributes, builder::nan_check, traits::Build};

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
        // [(0.0, 792.0), (10.0, 10.0), (10.0, 20.0)]
        // [Begin, End, Begin, LineTo, Close]
        self.verbs.insert(4, Verb::LineTo);
        self.verbs.insert(5, Verb::LineTo);

        let from = *self.points.get(1).unwrap();
        let to = *self.points.get(2).unwrap();
        let line = LineSegment { from, to };
        let rect_points = line.as_rect();
        self.points[1] = rect_points[0];
        self.points[2] = rect_points[1];
        self.points.push(rect_points[2]);
        self.points.push(rect_points[3]);
    }

}

impl Build for Pdf {
    type PathType = Path;

    fn build(mut self) -> Path
    {
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