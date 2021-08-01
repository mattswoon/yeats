use crate::game::{
    game_error::GameError,
    clue::Clue,
    player::Player,
};

#[derive(Clone, Debug)]
pub enum TurnState {
    Ready,
    Guessing,
    Ended
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

    pub fn as_guessing(self) -> Turn {
        Turn {
            performer: self.performer,
            guesser: self.guesser,
            state: TurnState::Guessing
        }
    }

    pub fn as_ended(self) -> Turn {
        Turn {
            performer: self.performer,
            guesser: self.guesser,
            state: TurnState::Ended
        }
    }

    pub fn status(&self) -> String {
        match self.state {
            TurnState::Ready => 
                format!("{} is getting ready to perform to {} who will be guessing", 
                        &self.performer, 
                        &self.guesser),
            TurnState::Guessing =>
                format!("{} is performing for {} who is guessing",
                        &self.performer,
                        &self.guesser),
            TurnState::Ended =>
                format!("{} has finished performing for {} who was guessing",
                        &self.performer,
                        &self.guesser)
        }
    }
}
