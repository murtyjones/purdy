use crate::events::{Event, PathEvent};
use crate::geom::{traits::Transformation, Arc, ArcFlags, LineSegment, SvgArc};
use crate::math::*;
use crate::path::Verb;
use crate::polygon::Polygon;
use crate::{EndpointId, Winding, Attributes};

use std::iter::IntoIterator;
use std::marker::Sized;

use crate::builder::{PathBuilder, Build, Flattened, Transformed, SvgPathBuilder, nan_check};


/// Implements an SVG-like building interface on top of a PathBuilder.
pub struct WithPdf<Builder: PathBuilder> {
    builder: Builder,

    first_position: Point,
    current_position: Point,
    last_ctrl: Point,
    last_cmd: Verb,
    need_moveto: bool,
    is_empty: bool,
    attribute_buffer: Vec<f32>,
}

impl<Builder: PathBuilder> WithPdf<Builder> {
    pub fn new(builder: Builder) -> Self {
        let attribute_buffer = vec![0.0; builder.num_attributes()];
        WithPdf {
            builder,
            first_position: point(0.0, 0.0),
            current_position: point(0.0, 0.0),
            last_ctrl: point(0.0, 0.0),
            need_moveto: true,
            is_empty: true,
            last_cmd: Verb::End,
            attribute_buffer,
        }
    }

    pub fn build(mut self) -> Builder::PathType
    where
        Builder: Build,
    {
        self.end_if_needed();
        self.builder.build()
    }

    pub fn flattened(self, tolerance: f32) -> WithPdf<Flattened<Builder>> {
        WithPdf::new(Flattened::new(self.builder, tolerance))
    }

    pub fn transformed<Transform>(
        self,
        transform: Transform,
    ) -> WithPdf<Transformed<Builder, Transform>>
    where
        Transform: Transformation<f32>,
    {
        WithPdf::new(Transformed::new(self.builder, transform))
    }

    pub fn move_to(&mut self, to: Point) -> EndpointId {
        self.end_if_needed();

        let id = self.builder.begin(to, Attributes(&self.attribute_buffer));

        self.is_empty = false;
        self.need_moveto = false;
        self.first_position = to;
        self.current_position = to;
        self.last_cmd = Verb::Begin;

        id
    }

    pub fn line_to(&mut self, to: Point) -> EndpointId {
        if let Some(id) = self.begin_if_needed(&to) {
            return id;
        }

        self.current_position = to;
        self.last_cmd = Verb::LineTo;

        self.builder.line_to(to, Attributes(&self.attribute_buffer))
    }

    pub fn close(&mut self) {
        if self.need_moveto {
            return;
        }

        // Relative path ops tend to accumulate small floating point imprecisions
        // which results in the last segment ending almost but not quite at the
        // start of the sub-path, causing a new edge to be inserted which often
        // intersects with the first or last edge. This can affect algorithms that
        // Don't handle self-intersecting paths.
        // Deal with this by snapping the last point if it is very close to the
        // start of the sub path.
        //
        // TODO
        // if let Some(p) = self.builder.points.last_mut() {
        //     let d = (*p - self.first_position).abs();
        //     if d.x + d.y < 0.0001 {
        //         *p = self.first_position;
        //     }
        // }

        self.current_position = self.first_position;
        self.need_moveto = true;
        self.last_cmd = Verb::Close;

        self.builder.close();
    }

    pub fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point) -> EndpointId {
        if let Some(id) = self.begin_if_needed(&to) {
            return id;
        }

        self.current_position = to;
        self.last_cmd = Verb::QuadraticTo;
        self.last_ctrl = ctrl;

        self.builder.quadratic_bezier_to(ctrl, to, Attributes(&self.attribute_buffer))
    }

    pub fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) -> EndpointId {
        if let Some(id) = self.begin_if_needed(&to) {
            return id;
        }

        self.current_position = to;
        self.last_cmd = Verb::CubicTo;
        self.last_ctrl = ctrl2;

        self.builder.cubic_bezier_to(ctrl1, ctrl2, to, Attributes(&self.attribute_buffer))
    }

    pub fn arc(&mut self, center: Point, radii: Vector, sweep_angle: Angle, x_rotation: Angle) {
        nan_check(center);
        nan_check(radii.to_point());
        debug_assert!(!sweep_angle.get().is_nan());
        debug_assert!(!x_rotation.get().is_nan());

        self.last_ctrl = self.current_position;

        // If the center is equal to the current position, the start and end angles aren't
        // defined, so we just skip the arc to avoid generating NaNs that will cause issues
        // later.
        use lyon_geom::euclid::approxeq::ApproxEq;
        if self.current_position.approx_eq(&center) {
            return;
        }

        let start_angle = (self.current_position - center).angle_from_x_axis() - x_rotation;

        let arc = Arc {
            center,
            radii,
            start_angle,
            sweep_angle,
            x_rotation,
        };

        // If the current position is not on the arc, move or line to the beginning of the
        // arc.
        let arc_start = arc.from();
        if self.need_moveto {
            self.move_to(arc_start);
        } else if (arc_start - self.current_position).square_length() < 0.01 {
            self.builder.line_to(arc_start, Attributes(&self.attribute_buffer));
        }

        arc.for_each_quadratic_bezier(&mut |curve| {
            self.builder.quadratic_bezier_to(curve.ctrl, curve.to, Attributes(&self.attribute_buffer));
            self.current_position = curve.to;
        });
    }

    /// Ensures the current sub-path has a moveto command.
    ///
    /// Returns an ID if the command should be skipped and the ID returned instead.
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

    fn end_if_needed(&mut self) {
        if (self.last_cmd as u8) <= (Verb::Begin as u8) {
            self.builder.end(false);
        }
    }

    pub fn current_position(&self) -> Point {
        self.current_position
    }

    pub fn reserve(&mut self, endpoints: usize, ctrl_points: usize) {
        self.builder.reserve(endpoints, ctrl_points);
    }

    fn get_smooth_cubic_ctrl(&self) -> Point {
        match self.last_cmd {
            Verb::CubicTo => self.current_position + (self.current_position - self.last_ctrl),
            _ => self.current_position,
        }
    }

    fn get_smooth_quadratic_ctrl(&self) -> Point {
        match self.last_cmd {
            Verb::QuadraticTo => self.current_position + (self.current_position - self.last_ctrl),
            _ => self.current_position,
        }
    }

    fn relative_to_absolute(&self, v: Vector) -> Point {
        self.current_position + v
    }
}

impl<Builder, Transform> WithPdf<Transformed<Builder, Transform>>
where
    Builder: PathBuilder,
    Transform: Transformation<f32>,
{
    #[inline]
    pub fn set_transform(&mut self, transform: Transform) {
        self.builder.set_transform(transform);
    }
}

impl<Builder: PathBuilder + Build> Build for WithPdf<Builder> {
    type PathType = Builder::PathType;

    fn build(mut self) -> Builder::PathType {
        self.end_if_needed();
        self.builder.build()
    }
}

impl<Builder: PathBuilder> SvgPathBuilder for WithPdf<Builder> {
    fn move_to(&mut self, to: Point) {
        self.move_to(to);
    }

    fn line_to(&mut self, to: Point) {
        self.line_to(to);
    }

    fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point) {
        self.quadratic_bezier_to(ctrl, to);
    }

    fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    fn close(&mut self) {
        self.close();
    }

    fn relative_move_to(&mut self, to: Vector) {
        let to = self.relative_to_absolute(to);
        self.move_to(to);
    }

    fn relative_line_to(&mut self, to: Vector) {
        let to = self.relative_to_absolute(to);
        self.line_to(to);
    }

    fn relative_quadratic_bezier_to(&mut self, ctrl: Vector, to: Vector) {
        let ctrl = self.relative_to_absolute(ctrl);
        let to = self.relative_to_absolute(to);
        self.quadratic_bezier_to(ctrl, to);
    }

    fn relative_cubic_bezier_to(&mut self, ctrl1: Vector, ctrl2: Vector, to: Vector) {
        let to = self.relative_to_absolute(to);
        let ctrl1 = self.relative_to_absolute(ctrl1);
        let ctrl2 = self.relative_to_absolute(ctrl2);
        self.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    fn smooth_cubic_bezier_to(&mut self, ctrl2: Point, to: Point) {
        let ctrl1 = self.get_smooth_cubic_ctrl();
        self.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    fn smooth_relative_cubic_bezier_to(&mut self, ctrl2: Vector, to: Vector) {
        let ctrl1 = self.get_smooth_cubic_ctrl();
        let ctrl2 = self.relative_to_absolute(ctrl2);
        let to = self.relative_to_absolute(to);
        self.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    fn smooth_quadratic_bezier_to(&mut self, to: Point) {
        let ctrl = self.get_smooth_quadratic_ctrl();
        self.quadratic_bezier_to(ctrl, to);
    }

    fn smooth_relative_quadratic_bezier_to(&mut self, to: Vector) {
        let ctrl = self.get_smooth_quadratic_ctrl();
        let to = self.relative_to_absolute(to);
        self.quadratic_bezier_to(ctrl, to);
    }

    fn horizontal_line_to(&mut self, x: f32) {
        let y = self.current_position.y;
        self.line_to(point(x, y));
    }

    fn relative_horizontal_line_to(&mut self, dx: f32) {
        let p = self.current_position;
        self.line_to(point(p.x + dx, p.y));
    }

    fn vertical_line_to(&mut self, y: f32) {
        let x = self.current_position.x;
        self.line_to(point(x, y));
    }

    fn relative_vertical_line_to(&mut self, dy: f32) {
        let p = self.current_position;
        self.line_to(point(p.x, p.y + dy));
    }

    fn arc_to(&mut self, radii: Vector, x_rotation: Angle, flags: ArcFlags, to: Point) {
        let svg_arc = SvgArc {
            from: self.current_position,
            to,
            radii,
            x_rotation,
            flags: ArcFlags {
                large_arc: flags.large_arc,
                sweep: flags.sweep,
            },
        };

        if svg_arc.is_straight_line() {
            self.line_to(to);
        } else {
            let arc = svg_arc.to_arc();
            self.arc(arc.center, arc.radii, arc.sweep_angle, arc.x_rotation);
        }
    }

    fn relative_arc_to(&mut self, radii: Vector, x_rotation: Angle, flags: ArcFlags, to: Vector) {
        let to = self.relative_to_absolute(to);
        self.arc_to(radii, x_rotation, flags, to);
    }

    fn reserve(&mut self, endpoints: usize, ctrl_points: usize) {
        self.builder.reserve(endpoints, ctrl_points);
    }
}
