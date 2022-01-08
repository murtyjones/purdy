use anyhow::{Result, Ok};
use anyhow::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum GraphicsStateError {
    #[error("invalid command: tried to convert PageDescription to PageDescription")]
    TriedToConvertPageDescriptionToPageDescription,
    #[error("invalid command: tried to convert PageDescription to ClippingPath")]
    TriedToConvertPageDescriptionToClippingPath,
    #[error("invalid command: tried to convert Text to Text")]
    TriedToConvertTextToText,
    #[error("invalid command: tried to convert Text to Path")]
    TriedToConvertTextToPath,
    #[error("invalid command: tried to convert Text to ClippingPath")]
    TriedToConvertTextToClippingPath,
    #[error("invalid command: tried to convert Path to Path")]
    TriedToConvertPathToPath,
    #[error("invalid command: tried to convert Path to Text")]
    TriedToConvertPathToText,
    #[error("invalid command: tried to convert ClippingPath to ClippingPath")]
    TriedToConvertClippingPathToClippingPath,
    #[error("invalid command: tried to convert ClippingPath to Path")]
    TriedToConvertClippingPathToPath,
    #[error("invalid command: tried to convert ClippingPath to Text")]
    TriedToConvertClippingPathToText,
}

#[derive(Debug, PartialEq)]
pub(crate) enum State {
    PageDescription(PageDescription),
    Text(Text),
    Path(Path),
    ClippingPath(ClippingPath),
}

impl Default for State {
    fn default() -> Self { 
        State::PageDescription(PageDescription {})
     }
}


#[derive(Debug, PartialEq)]
pub(crate) struct GraphicsState {
    // ... Shared Values
    state: State
}

#[derive(Debug, PartialEq)]
pub(crate) struct PageDescription {
    // ... Specific State Values
}

#[derive(Debug, PartialEq)]
pub(crate) struct Text {
    // ... Specific State Values
}

#[derive(Debug, PartialEq)]
pub(crate) struct Path {
    // ... Specific State Values
}

#[derive(Debug, PartialEq)]
pub(crate) struct ClippingPath {
    // ... Specific State Values
}

// Raft starts in the Path state
impl GraphicsState {
    pub fn new(/* ... */) -> Self {
        // ...
        GraphicsState {
            // ...
            state: State::default()
        }
    }

    fn to_page_description(self) -> Result<GraphicsState> {
        let result = match self.state {
            // TODO: Should this be an error or is it OK to try and turn a page_description into a page_description?
            //       "What Would Acrobat Do?""
            State::PageDescription(_) => {
                Err(GraphicsStateError::TriedToConvertPageDescriptionToPageDescription.into())
            }
            State::Text(data) => {
                Ok(GraphicsState {
                    state: State::PageDescription(convert_text_to_page_description(data))
                })
            }
            State::Path(data) => {
                Ok(GraphicsState {
                    state: State::PageDescription(convert_path_to_page_description(data))
                })
            }
            State::ClippingPath(data) => {
                Ok(GraphicsState {
                    state: State::PageDescription(convert_clipping_path_to_page_description(data))
                })
            }
        }?;
        assert!(matches!(result.state, State::PageDescription(_)));
        Ok(result)
    }

    fn to_text(self) -> Result<GraphicsState> {
        let result = match self.state {
            State::PageDescription(data) => {
                Ok(GraphicsState {
                    state: State::Text(convert_page_description_to_text(data))
                })
            }
            // TODO: Should this be an error or is it OK to try and turn a text into a text?
            //       "What Would Acrobat Do?""
            State::Text(_) => {
                Err(GraphicsStateError::TriedToConvertTextToText.into())
            }
            State::Path(_) => {
                Err(GraphicsStateError::TriedToConvertPathToText.into())
            }
            State::ClippingPath(_) => {
                Err(GraphicsStateError::TriedToConvertClippingPathToText.into())
            }
        }?;
        assert!(matches!(result.state, State::Text(_)));
        Ok(result)
    }

    fn to_path(self) -> Result<GraphicsState> {
        let result = match self.state {
            State::PageDescription(data) => {
                Ok(GraphicsState {
                    state: State::Path(convert_page_description_to_path(data))
                })
            }
            State::Text(data) => {
                Err(GraphicsStateError::TriedToConvertTextToPath.into())
            }
            // TODO: Should this be an error or is it OK to try and turn a path into a path?
            //       "What Would Acrobat Do?""
            State::Path(_) => {
                Err(GraphicsStateError::TriedToConvertPathToPath.into())
            }
            State::ClippingPath(_) => {
                Err(GraphicsStateError::TriedToConvertClippingPathToPath.into())
            }
        }?;
        assert!(matches!(result.state, State::Path(_)));
        Ok(result)
    }

    fn to_clipping_path(self) -> Result<GraphicsState> {
        let result = match self.state {
            State::PageDescription(_) => {
                Err(GraphicsStateError::TriedToConvertPageDescriptionToClippingPath.into())
            }
            State::Text(_) => {
                Err(GraphicsStateError::TriedToConvertTextToClippingPath.into())
            }
            State::Path(data) => {
                Ok(GraphicsState {
                    state: State::ClippingPath(convert_path_to_clipping_path(data))
                })
            }
            // TODO: Should this be an error or is it OK to try and turn a path into a path?
            //       "What Would Acrobat Do?""
            State::ClippingPath(_) => {
                Err(GraphicsStateError::TriedToConvertClippingPathToClippingPath.into())
            }
        }?;
        assert!(matches!(result.state, State::ClippingPath(_)));
        Ok(result)
    }
}

fn convert_text_to_page_description(data: Text) -> PageDescription {
    PageDescription {}
}

fn convert_path_to_page_description(data: Path) -> PageDescription {
    PageDescription {}
}

fn convert_clipping_path_to_page_description(data: ClippingPath) -> PageDescription {
    PageDescription {}
}

fn convert_page_description_to_text(data: PageDescription) -> Text {
    Text {}
}

fn convert_path_to_text(data: Path) -> Text {
    Text {}
}

fn convert_text_to_path(data: Text) -> Path {
    Path {}
}

fn convert_page_description_to_path(data: PageDescription) -> Path {
    Path {}
}

fn convert_path_to_clipping_path(data: Path) -> ClippingPath {
    ClippingPath {}
}


#[test]
fn test_foo() {
    let state = GraphicsState::new();
    let state = state.to_text().unwrap();
    let state = state.to_page_description().unwrap();
    let state = state.to_path().unwrap();
    let state = state.to_text().unwrap();
}