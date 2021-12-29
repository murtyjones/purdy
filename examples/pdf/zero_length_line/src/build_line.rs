#![allow(clippy::approx_constant)]
use lyon::geom::point;
use lyon::path::builder::SvgPathBuilder;

pub fn build_line<Builder: SvgPathBuilder>(path: &mut Builder) {
    path.move_to(point(10.0, 10.0));
    path.relative_vertical_line_to(0.0);
    path.close();
}

#[cfg(test)]
mod test {
    use lyon::path::Path;
    use super::*;

    #[test]
    fn build_zero_length_line() {
        // Build a Path for the rust logo.
        // TODO: with_pdf()
        let mut builder = Path::builder().with_svg();
        build_line(&mut builder);
        let path = builder.build();
        assert_eq!(format!("{:?}", path), "10.0 10.0 m 10.0 10.0 l f");
    }
}