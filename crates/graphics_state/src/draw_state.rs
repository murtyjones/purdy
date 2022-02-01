use anyhow::{Ok, Result};
use lyon::math::Point;
use strum_macros::Display;
use thiserror::Error;

#[derive(Debug, Copy, Clone)]
pub struct DrawState(State);

impl Default for DrawState {
    fn default() -> Self {
        DrawState(State::default())
    }
}

impl DrawState {
    pub fn current(&self) -> State {
        self.0
    }
    pub fn assert_is_inactive(&self) -> Result<()> {
        match &self.0 {
            State::Inactive => Ok(()),
            _ => Err(GraphicsStateError::AttemptToAccessWrongState("Inactive").into()),
        }
    }
    pub fn assert_is_not_inactive(&self) -> Result<()> {
        match &self.0 {
            State::Inactive => Err(GraphicsStateError::StateAssertion("Inactive").into()),
            _ => Ok(()),
        }
    }

    pub fn assert_is_active(&self) -> Result<Active> {
        match &self.0 {
            State::Active(s) => Ok(*s),
            _ => Err(GraphicsStateError::AttemptToAccessWrongState("Active").into()),
        }
    }

    pub fn assert_is_commands(&self) -> Result<&Commands> {
        match &self.0 {
            State::Commands(data) => Ok(data),
            _ => Err(GraphicsStateError::AttemptToAccessWrongState("Commands").into()),
        }
    }

    pub fn make_inactive(&mut self) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => Ok(()),
            State::Active(_) => {
                self.0 = State::Inactive;
                Ok(())
            }
            State::Commands(_) => {
                self.0 = State::Inactive;
                Ok(())
            }
        }?;
        assert!(matches!(self.0, State::Inactive));
        Ok(result)
    }

    pub fn make_active(&mut self, at: Point) -> Result<()> {
        let result = match &self.0 {
            State::Inactive => {
                self.0 = State::Active(Active::new(at));
                Ok(())
            }
            State::Active(_) => Ok(()),
            State::Commands(_data) => {
                Err(GraphicsStateError::StateTransition("Commands", "Active").into())
            }
        }?;
        assert!(matches!(self.0, State::Active(_)));
        Ok(result)
    }

    pub fn make_commands(&mut self, command: Command) -> Result<()> {
        let result = match self.0 {
            State::Inactive => {
                Err(GraphicsStateError::StateTransition("Inactive", "Commands").into())
            }
            State::Active(s) => {
                let c: Commands = s.into();
                let c = c.with_command(command);
                self.0 = State::Commands(c);
                Ok(())
            }
            State::Commands(current) => {
                self.0 = State::Commands(current.with_command(command));
                Ok(())
            }
        }?;
        assert!(matches!(self.0, State::Commands(_)));
        Ok(result)
    }
}

impl From<Active> for Commands {
    fn from(state: Active) -> Self {
        Commands::new(state.first)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Active {
    pub(crate) first: Point,
}

impl Active {
    pub fn new(first: Point) -> Self {
        Active { first }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Commands {
    pub(crate) line_to: bool,
    pub(crate) cubic_bezier: bool,
    pub(crate) quadratic_bezier: bool,
    pub(crate) current: Point,
    pub(crate) first: Point,
}

impl Commands {
    pub fn new(first: Point) -> Self {
        Commands {
            line_to: false,
            cubic_bezier: false,
            quadratic_bezier: false,
            current: first,
            first,
        }
    }

    pub fn with_command(self, command: Command) -> Self {
        match command {
            Command::CubicBezier => Commands {
                cubic_bezier: true,
                ..self
            },
            Command::QuadraticBezier => Commands {
                quadratic_bezier: true,
                ..self
            },
            Command::LineTo => Commands {
                line_to: true,
                ..self
            },
        }
    }
}

#[derive(Display, Debug)]
pub enum Command {
    LineTo,
    CubicBezier,
    QuadraticBezier,
}

#[derive(Debug, Copy, Clone, Display)]
pub enum State {
    Inactive,
    Active(Active),
    Commands(Commands),
}

impl Default for State {
    fn default() -> Self {
        State::Inactive
    }
}

#[derive(Error, Debug)]
pub(crate) enum GraphicsStateError {
    #[error("invalid state transition: tried to convert {0} to {1}")]
    StateTransition(&'static str, &'static str),
    #[error("invalid attempt to access {0} state while not in {0} mode")]
    AttemptToAccessWrongState(&'static str),
    #[error("is in state {0} but should not be")]
    StateAssertion(&'static str),
}

#[cfg(test)]
mod tests {
    use lyon::math::point;

    use super::{Command, Commands, DrawState};

    #[test]
    fn test_draw_state_transitions() {
        let mut state = DrawState::default();
        assert!(state.assert_is_inactive().is_ok());
        assert!(state.assert_is_active().is_err());
        assert!(state.assert_is_commands().is_err());
        assert!(state.make_inactive().is_ok());
        assert!(state.make_commands(Command::LineTo).is_err());
        assert!(state.make_active(point(0.0, 0.0)).is_ok());
        assert!(state.assert_is_active().is_ok());
        assert!(state.assert_is_inactive().is_err());
        assert!(state.assert_is_commands().is_err());
        assert!(state.make_commands(Command::LineTo).is_ok());
        assert!(state.assert_is_inactive().is_err());
        assert!(state.assert_is_active().is_err());
        assert!(state.assert_is_commands().is_ok());
        let given_commands = state.assert_is_commands().unwrap();
        assert_eq!(
            *given_commands,
            Commands::new(point(0.0, 0.0)).with_command(Command::LineTo)
        );
        assert!(state.make_commands(Command::QuadraticBezier).is_ok());
        assert!(state.assert_is_commands().is_ok());
        let given_commands = state.assert_is_commands().unwrap();
        assert_eq!(
            *given_commands,
            Commands::new(point(0.0, 0.0))
                .with_command(Command::QuadraticBezier)
                .with_command(Command::LineTo)
        );
        assert!(state.make_active(point(0.0, 0.0)).is_err());
        assert!(state.make_inactive().is_ok());
    }
}
