use std::fmt::{Display, Formatter, self};
use crate::game::{
    game_error::GameError,
    clue::Clue,
    player::Player,
};

#[derive(Clone, Debug)]
pub enum TurnState {
    Ready,
    Guessing(TurnSummary),
    Ended(TurnSummary),
}

#[derive(Clone, Debug)]
pub struct TurnSummary {
    clues_solved: Vec<Clue>
}

impl Display for TurnSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "Clues solved:\n{}", 
            self.clues_solved
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"))
    }
}

impl TurnSummary {
    pub fn new() -> TurnSummary {
        TurnSummary {
            clues_solved: vec![],
        }
    }

    pub fn with_clue(self, clue: Clue) -> TurnSummary {
        TurnSummary {
            clues_solved: self.clues_solved
                .into_iter()
                .chain(vec![clue])
                .collect()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Turn {
    pub performer: Player,
    pub guesser: Player,
    pub state: TurnState
}

impl Turn {
    pub fn new(p1: Player, p2: Player) -> Turn {
        Turn {
            performer: p1,
            guesser: p2,
            state: TurnState::Ready
        }
    }

    pub fn with_state(self, state: TurnState) -> Turn {
        Turn { state, ..self }
    }

    pub fn as_guessing(self) -> Turn {
        Turn {
            performer: self.performer,
            guesser: self.guesser,
            state: TurnState::Guessing(TurnSummary::new()),
        }
    }

    pub fn as_ended(self) -> Turn {
        match self.state {
            TurnState::Ready => {
                Turn {
                    performer: self.performer,
                    guesser: self.guesser,
                    state: TurnState::Ended(TurnSummary::new()),
                }
            },
            TurnState::Guessing(summ) | TurnState::Ended(summ) => {
                Turn {
                    performer: self.performer,
                    guesser: self.guesser,
                    state: TurnState::Ended(summ)
                }
            }
        }
    }

    pub fn status(&self) -> String {
        match self.state {
            TurnState::Ready => 
                format!("{} is getting ready to perform to {} who will be guessing", 
                        &self.performer, 
                        &self.guesser),
            TurnState::Guessing(_) =>
                format!("{} is performing for {} who is guessing",
                        &self.performer,
                        &self.guesser),
            TurnState::Ended(_) =>
                format!("{} has finished performing for {} who was guessing",
                        &self.performer,
                        &self.guesser)
        }
    }

    pub fn with_solved_clue(self, clue: Clue) -> Result<Self, GameError> {
        match self.state {
            TurnState::Ready | TurnState::Ended(_) => Err(GameError::BadTurnState(self.clone())),
            TurnState::Guessing(v) => Ok(Turn {
                performer: self.performer,
                guesser: self.guesser,
                state: TurnState::Guessing(v.with_clue(clue))
            })
        }
    }
}
