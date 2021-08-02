use crate::game::game_error::GameError;

#[derive(Debug)]
pub enum Error {
    NoGame,
    NoChannel,
    NotAGuildChannel,
    GameError(GameError),
    Serenity(serenity::prelude::SerenityError),
    Command(serenity::framework::standard::CommandError),
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
            Error::Serenity(e) => write!(f, "{}", e),
            Error::Command(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}
