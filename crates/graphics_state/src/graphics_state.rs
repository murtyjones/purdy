use anyhow::{Result, Ok};
use lyon_path::math::Vector;
use thiserror::Error;
use lyon_path::pdf::Pdf;
use lyon_path::builder::Build;
use shared::{PageWidth, PageHeight};

#[derive(Error, Debug)]
pub(crate) enum GraphicsStateError {
    #[error("invalid command: tried to convert PageDescription to ClippingPath")]
    TriedToConvertPageDescriptionToClippingPath,
    #[error("invalid command: tried to convert Text to Text")]
    TriedToConvertTextToText,
    #[error("invalid command: tried to convert Text to Path")]
    TriedToConvertTextToPath,
    #[error("invalid command: tried to convert Text to ClippingPath")]
    TriedToConvertTextToClippingPath,
    #[error("invalid command: tried to convert Path to Text")]
    TriedToConvertPathToText,
    #[error("invalid command: tried to convert ClippingPath to Path")]
    TriedToConvertClippingPathToPath,
    #[error("invalid command: tried to convert ClippingPath to Text")]
    TriedToConvertClippingPathToText,
    #[error("invalid attempt to access Path state while not in Path mode")]
    InvalidAttemptToAccessPathGraphicsState,
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
    pub finished_fill_paths: Vec<lyon_path::Path>,
    page_width: PageWidth,
    page_height: PageHeight,
    // ... Shared Values
    state: State
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

#[derive(Debug)]
struct Path {
    // ... Specific State Values
    builder: Pdf
}

impl Path {
    fn new(page_width: PageWidth, page_height: PageHeight) -> Self {
        Path {
            builder: Pdf::new(page_width, page_height)
        }
    }

    fn move_to(&mut self, to: Vector) {
        self.builder.move_to(to);
    }
    
    fn line_to(&mut self, to: Vector) {
        self.builder.line_to(to);
    }
    
    fn close(&mut self) {
        self.builder.close();
    }
    
    fn build(self) -> lyon_path::Path {
        self.builder.build()
    }

    fn make_fillable_if_needed(&mut self) {
        self.builder.make_fillable_if_needed();
    }
}

#[derive(Debug, PartialEq)]
struct ClippingPath {
    // ... Specific State Values
}

impl Default for ClippingPath {
    fn default() -> Self {
        ClippingPath::default()
    }
}

// Raft starts in the Path state
impl GraphicsState {
    pub fn new(page_width: PageWidth, page_height: PageHeight) -> Self {
        // ...
        GraphicsState {
            finished_fill_paths: vec![],
            page_width,
            page_height,
            // ...
            state: State::default()
        }
    }

    pub fn move_to(&mut self, to: Vector) -> Result<()> {
        self.to_path()?;
        self.as_path()?.move_to(to);
        Ok(())
    }

    pub fn line_to(&mut self, to: Vector) -> Result<()> {
        self.to_path()?;
        self.as_path()?.line_to(to);
        Ok(())
    }

    pub fn fill(&mut self) -> Result<()> {
        self.to_path()?;
        let w = self.page_width;
        let h = self.page_height;
        let mut p = std::mem::replace(self.as_path()?, Path::new(w, h));
        p.close();
        p.make_fillable_if_needed();
        let path = p.build();
        self.finished_fill_paths.push(path);
        self.to_page_description()?;
        Ok(())
    }

    fn as_path(&mut self) -> Result<&mut Path> {
        match &mut self.state {
            State::Path(data) => Ok(data),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessPathGraphicsState.into())
        }
    }

    fn to_page_description(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(_) => {
                Ok(())
            }
            State::Text(data) => {
                self.state = State::PageDescription(convert_text_to_page_description(data));
                Ok(())
            }
            State::Path(data) => {
                self.state = State::PageDescription(convert_path_to_page_description(data));
                Ok(())
            }
            State::ClippingPath(data) => {
                self.state = State::PageDescription(convert_clipping_path_to_page_description(data));
                Ok(())
            }
        }?;
        assert!(matches!(self.state, State::PageDescription(_)));
        Ok(result)
    }

    fn to_text(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(data) => {
                self.state = State::Text(convert_page_description_to_text(&data));
                Ok(())
            }
            State::Text(_) => {
                Ok(())
            }
            State::Path(_) => {
                Err(GraphicsStateError::TriedToConvertPathToText.into())
            }
            State::ClippingPath(_) => {
                Err(GraphicsStateError::TriedToConvertClippingPathToText.into())
            }
        }?;
        assert!(matches!(self.state, State::Text(_)));
        Ok(result)
    }

    fn to_path(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(data) => {
                self.state = State::Path(convert_page_description_to_path(self.page_width, self.page_height, data));
                Ok(())
            }
            State::Text(_) => {
                Err(GraphicsStateError::TriedToConvertTextToPath.into())
            }
            State::Path(_) => {
                Ok(())
            }
            State::ClippingPath(_) => {
                Err(GraphicsStateError::TriedToConvertClippingPathToPath.into())
            }
        }?;
        assert!(matches!(self.state, State::Path(_)));
        Ok(result)
    }

    fn to_clipping_path(&mut self) -> Result<()> {
        let result = match &self.state {
            State::PageDescription(_) => {
                Err(GraphicsStateError::TriedToConvertPageDescriptionToClippingPath.into())
            }
            State::Text(_) => {
                Err(GraphicsStateError::TriedToConvertTextToClippingPath.into())
            }
            State::Path(data) => {
                self.state = State::ClippingPath(convert_path_to_clipping_path(&data));
                Ok(())
            }
            State::ClippingPath(_) => {
                Ok(())
            }
        }?;
        assert!(matches!(self.state, State::ClippingPath(_)));
        Ok(result)
    }
}

fn convert_text_to_page_description(data: &Text) -> PageDescription {
    PageDescription::default()
}

fn convert_path_to_page_description(data: &Path) -> PageDescription {
    PageDescription::default()
}

fn convert_clipping_path_to_page_description(data: &ClippingPath) -> PageDescription {
    PageDescription::default()
}

fn convert_page_description_to_text(data: &PageDescription) -> Text {
    Text::default()
}

fn convert_path_to_text(data: &Path) -> Text {
    Text::default()
}

fn convert_page_description_to_path(page_width: PageWidth, page_height: PageHeight, data: &PageDescription) -> Path {
    Path::new(page_width, page_height)
}

fn convert_path_to_clipping_path(data: &Path) -> ClippingPath {
    ClippingPath::default()
}
