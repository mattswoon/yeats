
#[derive(Debug)]
pub enum GameError {
    CantDoThat,
    AlreadyStarted
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
