use tokio::time::{Duration, delay_for};
use crate::game::{
    game_error::GameError,
    clue::Clue,
    player::Player,
};

#[derive(Clone)]
pub enum TurnState {
    Ready,
    Guessing,
    Ended
}

#[derive(Clone)]
pub struct Turn {
    performer: Player,
    guesser: Player,
    state: TurnState
}

impl Turn {
    pub fn new(p1: Player, p2: Player) -> Turn {
        Turn {
            performer: p1,
            guesser: p2,
            state: TurnState::Ready
        }
    }

    pub async fn start(&mut self) {
        self.state = TurnState::Guessing;
        delay_for(Duration::new(60, 0)).await;
        self.end();
    }

    pub fn end(&mut self) {
        self.state = TurnState::Ended;
    }
}
