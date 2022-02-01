use crate::path::Path;
use anyhow::{Ok, Result};
use lyon::path::{math::Vector, LineCap, PathEvent};

use shared::{
    Color, ColorSpace, DashPattern, Height, LineWidth, NonStrokeColor, StrokeColor, Width,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum GraphicsStateError {
    #[error("invalid state transition: tried to convert {0} to {1}")]
    InvalidStateTransition(&'static str, &'static str),
    #[error("invalid attempt to access {0} state while not in {0} mode")]
    InvalidAttemptToAccessState(&'static str),
}

#[derive(Debug)]
enum State {
    PageDescription(PageDescription),
    Text(Text),
    Path(Path),
    ClippingPath(ClippingPath),
}

impl Default for State {
    fn default() -> Self {
        State::PageDescription(PageDescription::default())
    }
}

#[derive(Debug)]
pub struct GraphicsState {
    pub properties: Properties,
    page_width: Width,
    page_height: Height,
    // ... Shared Values
    state: State,
}

#[derive(Debug, Clone)]
pub struct Properties {
    pub line_width: LineWidth,
    pub line_cap: LineCap,
    pub dash_pattern: DashPattern,
    pub stroke_color: StrokeColor,
    pub non_stroke_color: NonStrokeColor,
}

impl Default for Properties {
    fn default() -> Self {
        Properties {
            line_width: LineWidth::default(),
            line_cap: LineCap::Square,
            dash_pattern: DashPattern::default(),
            stroke_color: StrokeColor::default(),
            non_stroke_color: NonStrokeColor::default(),
        }
    }
}

#[derive(Debug)]
struct PageDescription {
    // ... Specific State Values
}

impl Default for PageDescription {
    fn default() -> Self {
        PageDescription {}
    }
}

#[derive(Debug)]
struct Text {
    // ... Specific State Values
}

impl Default for Text {
    fn default() -> Self {
        Text {}
    }
}
#[derive(Debug, PartialEq)]
struct ClippingPath {
    // ... Specific State Values
}

impl Default for ClippingPath {
    fn default() -> Self {
        ClippingPath {}
    }
}

// Raft starts in the Path state
impl GraphicsState {
    pub fn new(page_width: Width, page_height: Height) -> Self {
        GraphicsState {
            properties: Properties::default(),
            page_width,
            page_height,
            // ...
            state: State::default(),
        }
    }

    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    pub fn move_to(&mut self, to: Vector) -> Result<()> {
        self.path()?;
        self.assert_is_path_mut()?.move_to(to)?;
        // TODO: Implement
        Ok(())
    }

    pub fn line_to(&mut self, to: Vector) -> Result<()> {
        self.path()?;
        self.assert_is_path_mut()?.line_to(to)?;
        Ok(())
    }

    pub fn rect(&mut self, low_left: Vector, width: Width, height: Height) -> Result<()> {
        self.path()?;
        self.assert_is_path_mut()?.rect(low_left, width, height)?;
        Ok(())
    }

    fn take_path(&mut self) -> Result<Path> {
        let w = self.page_width;
        let h = self.page_height;
        Ok(std::mem::replace(
            self.assert_is_path_mut()?,
            Path::new(w, h),
        ))
    }

    pub fn fill(&mut self) -> Result<Vec<PathEvent>> {
        self.path()?;
        let mut path = self.take_path()?;
        path.close()?;
        path.make_fillable_if_needed();
        let paths = path.build()?;
        self.page_description()?;
        Ok(paths)
    }

    pub fn stroke(&mut self) -> Result<Vec<PathEvent>> {
        self.path()?;
        let mut path = self.take_path()?;
        path.close()?;
        let paths = path.build()?;
        self.page_description()?;
        Ok(paths)
    }

    pub fn set_line_width(&mut self, w: LineWidth) -> Result<()> {
        self.page_description()?;
        self.properties.line_width.set(w);
        Ok(())
    }

    pub fn set_non_stroke_color(&mut self, c: Vec<f32>) -> Result<()> {
        self.page_description()?;
        self.properties.non_stroke_color.set_color(c)
    }

    pub fn set_stroke_color(&mut self, c: Vec<f32>) -> Result<()> {
        self.page_description()?;
        self.properties.stroke_color.set_color(c)
    }

    pub fn set_non_stroke_color_space(&mut self, c: ColorSpace) -> Result<()> {
        self.page_description()?;
        self.properties.non_stroke_color.set_color_space(c);
        Ok(())
    }

    pub fn set_stroke_color_space(&mut self, c: ColorSpace) -> Result<()> {
        self.page_description()?;
        self.properties.stroke_color.set_color_space(c);
        Ok(())
    }

    pub fn set_cap_style(&mut self, c: LineCap) -> Result<()> {
        self.page_description()?;
        self.properties.line_cap = c;
        Ok(())
    }

    pub fn set_dash_pattern(&mut self, d: DashPattern) -> Result<()> {
        self.page_description()?;
        self.properties.dash_pattern = d;
        Ok(())
    }

    fn assert_is_path_mut(&mut self) -> Result<&mut Path> {
        match &mut self.state {
            State::Path(data) => Ok(data),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessState("Path").into()),
        }
    }

    fn assert_is_page_description_mut(&mut self) -> Result<&mut PageDescription> {
        match &mut self.state {
            State::PageDescription(data) => Ok(data),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessState("PageDescription").into()),
        }
    }

    fn assert_is_path(&self) -> Result<&Path> {
        match &self.state {
            State::Path(data) => Ok(data),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessState("Path").into()),
        }
    }

    fn assert_is_page_description(&self) -> Result<&PageDescription> {
        match &self.state {
            State::PageDescription(data) => Ok(data),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessState("PageDescription").into()),
        }
    }

    fn page_description(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(_) => Ok(()),
            State::Text(t) => {
                self.state = State::PageDescription(t.into());
                Ok(())
            }
            State::Path(p) => {
                self.state = State::PageDescription(p.into());
                Ok(())
            }
            State::ClippingPath(c) => {
                self.state = State::PageDescription(c.into());
                Ok(())
            }
        }?;
        assert!(matches!(self.state, State::PageDescription(_)));
        Ok(result)
    }

    fn text(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(p) => {
                self.state = State::Text(p.into());
                Ok(())
            }
            State::Text(_) => Ok(()),
            State::Path(_) => {
                Err(GraphicsStateError::InvalidStateTransition("Path", "Text").into())
            }
            State::ClippingPath(_) => {
                Err(GraphicsStateError::InvalidStateTransition("ClippingPath", "Text").into())
            }
        }?;
        assert!(matches!(self.state, State::Text(_)));
        Ok(result)
    }

    fn path(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(data) => {
                self.state = State::Path(convert_page_description_to_path(
                    self.page_width,
                    self.page_height,
                    data,
                ));
                Ok(())
            }
            State::Text(_) => {
                Err(GraphicsStateError::InvalidStateTransition("Text", "Path").into())
            }
            State::Path(_) => Ok(()),
            State::ClippingPath(_) => {
                Err(GraphicsStateError::InvalidStateTransition("ClippingPath", "Path").into())
            }
        }?;
        assert!(matches!(self.state, State::Path(_)));
        Ok(result)
    }

    fn clipping_path(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(_) => Err(GraphicsStateError::InvalidStateTransition(
                "PageDescription",
                "ClippingPath",
            )
            .into()),
            State::Text(_) => {
                Err(GraphicsStateError::InvalidStateTransition("ClippingPath", "Text").into())
            }
            State::Path(p) => {
                self.state = State::ClippingPath(p.into());
                Ok(())
            }
            State::ClippingPath(_) => Ok(()),
        }?;
        assert!(matches!(self.state, State::ClippingPath(_)));
        Ok(result)
    }
}

impl From<&Text> for PageDescription {
    fn from(_p: &Text) -> Self {
        PageDescription::default()
    }
}

impl From<&Path> for PageDescription {
    fn from(_p: &Path) -> Self {
        PageDescription::default()
    }
}

impl From<&ClippingPath> for PageDescription {
    fn from(_p: &ClippingPath) -> Self {
        PageDescription::default()
    }
}

impl From<&PageDescription> for Text {
    fn from(_p: &PageDescription) -> Self {
        Text::default()
    }
}

fn convert_page_description_to_path(
    page_width: Width,
    page_height: Height,
    _data: &PageDescription,
) -> Path {
    Path::new(page_width, page_height)
}

impl From<&Path> for ClippingPath {
    fn from(_p: &Path) -> Self {
        ClippingPath::default()
    }
}
