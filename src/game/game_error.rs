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
    PlayerNotAllowedToDrawAClue(Player),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::AlreadyStarted => 
                write!(f, "The game has already started"),
            GameError::PlayerNotAllowedToDrawAClue(p) =>
                write!(f, "{} is not allowed to draw a clue right now", p.user.name),
            GameError::CantDoThat =>
                write!(f, "I can't let you do that"),
            GameError::BadTurnState(t) => 
                write!(f, "Can't do that right now, because the turn state is {:?}", t.state),
            GameError::NoTurnsQueued =>
                write!(f, "There isn't a turn queued up yet"),
            GameError::TurnDoesntMatchPlayers { turn, performer, guesser } => 
                write!(f, 
                       "Current turn ({:?}) doesn't matcher the performer ({}) and guesser ({})",
                       turn, performer, guesser),
        }
    }
}
