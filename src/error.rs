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
    NoRound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoGame => 
                write!(f, "No Game object in context, this bad... real bad..."),
            Error::NoChannel =>
                write!(f, "No channel to play the game in, it should have been set when you ran `!start-game`"),
            Error::NotAGuildChannel =>
                write!(f, "Channel isn't a guild channel, run `!start-game` in #general or something like that"),
            Error::GameError(ge) => write!(f, "{}", ge),
            Error::GameAlreadyStarted =>
                write!(f, "I'm afraid I can't let you do that Dave, the game has already started"),
            Error::GameNotStartedYet =>
                write!(f, "The game hasn't started yet, to start the game run `!start-game`"),
            Error::GameFinished =>
                write!(f, "The game has finished, you can reset it with `!reset` if you want to play again"),
            Error::EmptyTurnQueue =>
                write!(f, "The turn queue is empty, something's gone horribly wrong, ask Tom"),
            Error::CurrentTurnNotYetFinished =>
                write!(f, "There's still a turn in progress, wait until that turn has finished to do this"),
            Error::CurrentTurnHasEnded => 
                write!(f, "The current turn has ended, you can queue up the next turn with `!next-turn`"),
            Error::CurrentTurnNotYetStarted =>
                write!(f, "The current turn hasn't started yet, you can start it with `!start-turn`"),
            Error::NoTurnsQueued =>
                write!(f, "No turns are queued, you can queue up the next turn with `!next-turn`"),
            Error::TurnDoesntMatchPlayers =>
                write!(f, "Turn doesn't match players or round"),
            Error::PlayerNotAllowedToDrawAClue =>
                write!(f, "Sorry, you're not allowed to draw a clue because it's not your turn"),
            Error::EmptyBowl =>
                write!(f, "Bowl is empty, it's time for the next round. Use `!next-round` to progress to the next round"),
            Error::BowlNotEmpty =>
                write!(f, "Can't go to the next round yet because the bowl isn't empty, queue up another turn instead with `!next-turn`"),
            Error::NoRound =>
                write!(f, "Game is not currently in any round, it's either finished or hasn't started yet. To start it, run `!start-game` or `!reset` if you want to play again")
        }
    }
}

impl std::error::Error for Error {}
