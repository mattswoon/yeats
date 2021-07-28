use tokio::time::{Duration, sleep};
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
        sleep(Duration::new(60, 0)).await;
        self.end();
    }

    pub fn end(&mut self) {
        self.state = TurnState::Ended;
    }
}
