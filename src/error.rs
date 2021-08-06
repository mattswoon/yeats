use crate::game::game_error::GameError;

#[derive(Debug, Clone)]
pub enum Error {
    NoGame,
    NoChannel,
    NotAGuildChannel,
    GameError(GameError),
    GameAlreadyStarted,
    GameNotStartedYet,
    GameFinished,
    EmptyTurnQueue,
    CurrentTurnNotYetFinished,
    CurrentTurnNotYetStarted,
    CurrentTurnHasEnded,
    NoTurnsQueued,
    TurnDoesntMatchPlayers,
    PlayerNotAllowedToDrawAClue,
    EmptyBowl,
    BowlNotEmpty,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoGame => 
                write!(f, "No Game object in context"),
            Error::NoChannel =>
                write!(f, "No channel to play the game in"),
            Error::NotAGuildChannel =>
                write!(f, "Channel isn't a guild channel"),
            Error::GameError(ge) => write!(f, "{}", ge),
            Error::GameAlreadyStarted =>
                write!(f, "I'm afraid I can't let you do that Dave, the game has already started"),
            Error::GameNotStartedYet =>
                write!(f, "The game hasn't started yet"),
            Error::GameFinished =>
                write!(f, "The game has finished"),
            Error::EmptyTurnQueue =>
                write!(f, "The turn queue is empty, something's gone horribly wrong, ask Tom"),
            Error::CurrentTurnNotYetFinished =>
                write!(f, "There's still a turn in progress"),
            Error::CurrentTurnHasEnded => 
                write!(f, "The current turn has ended"),
            Error::CurrentTurnNotYetStarted =>
                write!(f, "The current turn hasn't started yet"),
            Error::NoTurnsQueued =>
                write!(f, "No turns are queued"),
            Error::TurnDoesntMatchPlayers =>
                write!(f, "Turn doesn't match players"),
            Error::PlayerNotAllowedToDrawAClue =>
                write!(f, "Not allowed to draw a clue"),
            Error::EmptyBowl =>
                write!(f, "Bowl is empty"),
            Error::BowlNotEmpty =>
                write!(f, "Bowl isn't empty"),
        }
    }
}

impl std::error::Error for Error {}
