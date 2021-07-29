use crate::game::{
    turn::Turn,
    player::Player,
};

#[derive(Debug)]
pub enum GameError {
    CantDoThat,
    AlreadyStarted,
    BadTurnState(Turn),
    NoTurnsQueued,
    TurnDoesntMatchPlayers { turn: Turn, performer: Player, guesser: Player },
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::AlreadyStarted => 
                write!(f, "The game has already started"),
            _ => write!(f, "Some shit gone whoopsie")
        }
    }
}
