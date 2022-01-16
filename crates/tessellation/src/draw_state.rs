use anyhow::{Result, Ok};
use thiserror::Error;

#[derive(Debug)]
pub struct DrawState(State);

impl DrawState {
    fn to_inactive(&mut self) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => {
                Ok(())
            }
            State::Begun => {
                self.0 = State::Inactive;
                Ok(())
            }
            State::HasGivenCommands(_) => {
                self.0 = State::Inactive;
                Ok(())
            }
        }?;
        assert!(matches!(self.0, State::Inactive));
        Ok(result)
    }

    fn to_begun(&mut self) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => {
                self.0 = State::Begun;
                Ok(())
            }
            State::Begun => {
                Ok(())
            }
            State::HasGivenCommands(data) => {
                Err(GraphicsStateError::InvalidStateTransition("HasGivenCommands", "Begun").into())
            }
        }?;
        assert!(matches!(self.0, State::Begun));
        Ok(result)
    }

    fn to_has_given_commands(&mut self, command: Command) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => {
                Err(GraphicsStateError::InvalidStateTransition("Inactive", "HasGivenCommands").into())
            }
            State::Begun => {
                self.0 = State::HasGivenCommands(DrawState::generate_command_state(command, None));
                Ok(())
            }
            State::HasGivenCommands(data) => {
                self.0 = State::HasGivenCommands(DrawState::generate_command_state(command, Some(*data)));
                Ok(())
            }
        }?;
        assert!(matches!(self.0, State::HasGivenCommands(_)));
        Ok(result)
    }

    fn generate_command_state(new_command: Command, current_commands: Option<Commands>) -> Commands {
        let mut commands = current_commands.unwrap_or(Commands::default());
        match new_command {
            Command::LineTo => {
                commands.line_to = true;
            }
            Command::CubicBezier => {
                commands.cubic_bezier = true;
            }
            Command::QuadraticBezier => {
                commands.quadratic_bezier = true;
            }
        }
        commands
    }
}


#[derive(Debug)]
struct Commands {
    line_to: bool,
    cubic_bezier: bool,
    quadratic_bezier: bool,
}

impl Default for Commands {
    fn default() -> Self { 
        Commands {
            line_to: false,
            cubic_bezier: false,
            quadratic_bezier: false,
        }
     }
}

enum Command {
    LineTo,
    CubicBezier,
    QuadraticBezier,
}

#[derive(Debug)]
enum State {
    Inactive,
    Begun,
    HasGivenCommands(Commands),
}

impl Default for State {
    fn default() -> Self { 
        State::Inactive
     }
}

#[derive(Error, Debug)]
pub(crate) enum GraphicsStateError {
    #[error("invalid state transition: tried to convert {0} to {1}")]
    InvalidStateTransition(&'static str, &'static str),
    #[error("invalid attempt to access {0} state while not in {0} mode")]
    InvalidAttemptToAccessState(&'static str),
}