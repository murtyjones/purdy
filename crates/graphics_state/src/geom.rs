use lyon::geom::{
    point as s_point, Angle, LineSegment, Point as SPoint, Rotation, Scalar, Translation,
};

/// Converts the line to a simple, single-unit wide rect.
/// This is needed for PDF rendering in cases where there
/// exists a line followed by the fill command.
// TODO: Scaling might need to be handled
pub fn as_rect<S: Scalar>(line: LineSegment<S>) -> [SPoint<S>; 4] {
    let fill_width = S::ONE;
    let from = line.from;
    let to = line.to;
    let hypotenuse = get_pythagorean_hypotenuse(from, to);
    let p1 = s_point(-fill_width / S::TWO, hypotenuse / S::TWO);
    let p2 = s_point(fill_width / S::TWO, hypotenuse / S::TWO);
    let p3 = s_point(fill_width / S::TWO, -hypotenuse / S::TWO);
    let p4 = s_point(-fill_width / S::TWO, -hypotenuse / S::TWO);

    let x_mid = (from.x + to.x) / S::TWO;
    let y_mid = (from.y + to.y) / S::TWO;
    let degrees = S::atan2(to.y - from.y, to.x - from.x).to_degrees() - (S::NINE * S::TEN);
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

#[inline]
pub fn get_pythagorean_hypotenuse<S: Scalar>(p1: SPoint<S>, p2: SPoint<S>) -> S {
    let a_squared = (p1.y - p2.y).abs().powi(2);
    let b_squared = (p1.x - p2.x).abs().powi(2);
    let c_squared = a_squared + b_squared;
    c_squared.sqrt()
}
