use anyhow::{Result, Ok};
use anyhow::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum GraphicsStateError {
    #[error("invalid command: tried to convert follower to leader")]
    TriedToConvertFollowerToLeader,
    #[error("invalid command: tried to convert follower to follower")]
    TriedToConvertFollowerToFollower,
    #[error("invalid command: tried to convert leader to leader")]
    TriedToConvertLeaderToLeader,
    #[error("invalid command: tried to convert leader to candidate")]
    TriedToConvertLeaderToCandidate,
    #[error("invalid command: tried to convert candidate to candidate")]
    TriedToConvertCandidateToCandidate,
}

#[derive(Debug, PartialEq)]
pub(crate) enum State {
    Leader(Leader),
    Candidate(Candidate),
    Follower(Follower),
}

impl Default for State {
    fn default() -> Self { 
        State::Follower(Follower {})
     }
}


#[derive(Debug, PartialEq)]
pub(crate) struct Raft {
    // ... Shared Values
    state: State
}

#[derive(Debug, PartialEq)]
pub(crate) struct Leader {
    // ... Specific State Values
}

#[derive(Debug, PartialEq)]
pub(crate) struct Candidate {
    // ... Specific State Values
}

#[derive(Debug, PartialEq)]
pub(crate) struct Follower {
    // ... Specific State Values
}

// Raft starts in the Follower state
impl Raft {
    pub fn new(/* ... */) -> Self {
        // ...
        Raft {
            // ...
            state: State::default()
        }
    }

    pub fn to_leader(self) -> Result<Raft> {
        let result = match self.state {
            State::Candidate(data) => {
                Ok(Raft {
                    state: State::Leader(convert_candidate_to_leader(data))
                })
            }
            State::Follower(_) => {
                Err(GraphicsStateError::TriedToConvertFollowerToLeader.into())
            }
            // TODO: Should this be an error or is it OK to try and turn a leader into a leader?
            //       "What Would Acrobat Do?""
            State::Leader(_) => {
                Err(GraphicsStateError::TriedToConvertLeaderToLeader.into())
            }
        }?;
        assert!(matches!(result.state, State::Leader(_)));
        Ok(result)
    }

    pub fn to_candidate(self) -> Result<Raft> {
        let result = match self.state {
            // TODO: Should this be an error or is it OK to try and turn a candidate into a candidate?
            //       "What Would Acrobat Do?""
            State::Candidate(_) => {
                Err(GraphicsStateError::TriedToConvertCandidateToCandidate.into())
            }
            State::Follower(data) => {
                Ok(Raft {
                    state: State::Candidate(convert_follower_to_candidate(data))
                })
            }
            State::Leader(_) => {
                Err(GraphicsStateError::TriedToConvertLeaderToCandidate.into())
            }
        }?;
        assert!(matches!(result.state, State::Candidate(_)));
        Ok(result)
    }

    pub fn to_follower(self) -> Result<Raft> {
        let result = match self.state {
            State::Candidate(data) => {
                Ok(Raft {
                    state: State::Follower(convert_candidate_to_follower(data))
                })
            }
            // TODO: Should this be an error or is it OK to try and turn a follower into a follower?
            //       "What Would Acrobat Do?""
            State::Follower(_) => {
                Err(GraphicsStateError::TriedToConvertFollowerToFollower.into())
            }
            State::Leader(data) => {
                Ok(Raft {
                    state: State::Follower(convert_leader_to_follower(data))
                })
            }
        }?;
        assert!(matches!(result.state, State::Follower(_)));
        Ok(result)
    }
}

fn convert_candidate_to_leader(data: Candidate) -> Leader {
    Leader {}
}

fn convert_follower_to_candidate(data: Follower) -> Candidate {
    Candidate {}
}

fn convert_candidate_to_follower(data: Candidate) -> Follower {
    Follower {}
}

fn convert_leader_to_follower(data: Leader) -> Follower {
    Follower {}
}


#[test]
fn test_foo() {
    let state = Raft::new();
    let state = state.to_candidate().unwrap();
    let state = state.to_leader().unwrap();
    let state = state.to_follower().unwrap();
    let state = state.to_candidate().unwrap();
}