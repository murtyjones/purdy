use anyhow::{Ok, Result};
use thiserror::Error;

#[derive(Debug, Copy, Clone)]
pub struct DrawState(State);

impl Default for DrawState {
    fn default() -> Self {
        DrawState(State::default())
    }
}

impl DrawState {
    pub fn as_inactive(&self) -> Result<()> {
        match &self.0 {
            State::Inactive => Ok(()),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessState("Inactive").into()),
        }
    }

    pub fn as_begun(&self) -> Result<()> {
        match &self.0 {
            State::Begun => Ok(()),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessState("Begun").into()),
        }
    }

    pub fn as_has_given_commands(&self) -> Result<&Commands> {
        match &self.0 {
            State::HasGivenCommands(data) => Ok(data),
            _ => Err(GraphicsStateError::InvalidAttemptToAccessState("HasGivenCommands").into()),
        }
    }

    pub fn to_inactive(&mut self) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => Ok(()),
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

    pub fn to_begun(&mut self) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => {
                self.0 = State::Begun;
                Ok(())
            }
            State::Begun => Ok(()),
            State::HasGivenCommands(data) => {
                Err(GraphicsStateError::InvalidStateTransition("HasGivenCommands", "Begun").into())
            }
        }?;
        assert!(matches!(self.0, State::Begun));
        Ok(result)
    }

    pub fn to_has_given_commands(&mut self, command: Command) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => Err(GraphicsStateError::InvalidStateTransition(
                "Inactive",
                "HasGivenCommands",
            )
            .into()),
            State::Begun => {
                self.0 = State::HasGivenCommands(DrawState::generate_command_state(command, None));
                Ok(())
            }
            State::HasGivenCommands(data) => {
                self.0 = State::HasGivenCommands(DrawState::generate_command_state(
                    command,
                    Some(*data),
                ));
                Ok(())
            }
        }?;
        assert!(matches!(self.0, State::HasGivenCommands(_)));
        Ok(result)
    }

    fn generate_command_state(
        new_command: Command,
        current_commands: Option<Commands>,
    ) -> Commands {
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Commands {
    pub(crate) line_to: bool,
    pub(crate) cubic_bezier: bool,
    pub(crate) quadratic_bezier: bool,
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

pub enum Command {
    LineTo,
    CubicBezier,
    QuadraticBezier,
}

#[derive(Debug, Copy, Clone)]
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

#[cfg(test)]
mod tests {
    use super::{Command, Commands, DrawState};

    #[test]
    fn test_draw_state_transitions() {
        let mut state = DrawState::default();
        assert!(state.as_inactive().is_ok());
        assert!(state.as_begun().is_err());
        assert!(state.as_has_given_commands().is_err());
        assert!(state.to_inactive().is_ok());
        assert!(state.to_has_given_commands(Command::LineTo).is_err());
        assert!(state.to_begun().is_ok());
        assert!(state.as_begun().is_ok());
        assert!(state.as_inactive().is_err());
        assert!(state.as_has_given_commands().is_err());
        assert!(state.to_has_given_commands(Command::LineTo).is_ok());
        assert!(state.as_inactive().is_err());
        assert!(state.as_begun().is_err());
        assert!(state.as_has_given_commands().is_ok());
        let given_commands = state.as_has_given_commands().unwrap();
        assert_eq!(
            *given_commands,
            Commands {
                line_to: true,
                ..Commands::default()
            }
        );
        assert!(state
            .to_has_given_commands(Command::QuadraticBezier)
            .is_ok());
        assert!(state.as_has_given_commands().is_ok());
        let given_commands = state.as_has_given_commands().unwrap();
        assert_eq!(
            *given_commands,
            Commands {
                line_to: true,
                quadratic_bezier: true,
                ..Commands::default()
            }
        );
        assert!(state.to_begun().is_err());
        assert!(state.to_inactive().is_ok());
    }
}
