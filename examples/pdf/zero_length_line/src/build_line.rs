#![allow(clippy::approx_constant)]
use crate::commands::{Command, Commands};
use lyon::geom::point;
use lyon::path::builder::{SvgPathBuilder, PathBuilder};

pub fn build_line<Builder: SvgPathBuilder>(path: &mut Builder) {
    path.move_to(point(10.0, 10.0));
    path.relative_vertical_line_to(1.0);
    path.close();
}

pub fn build_graphics<Builder: SvgPathBuilder>(
    page_width: f32,
    page_height: f32,
    commands: Commands,
    path: &mut Builder,
) {
    // PDFs start at bottom left of page by default
    path.move_to(point(0.0, page_height));
    commands.0.iter().for_each(|e| match e {
        Command::MoveTo(x, y) => {
            path.move_to(point(*x, *y));
        }
        Command::LineTo(x, y) => {
            path.line_to(point(*x, *y));
        }
        Command::Fill => {
            path.close();
        }
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use lyon::path::Path;

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
