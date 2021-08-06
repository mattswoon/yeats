use crate::game::{
    player::Player,
};
use std::fmt::{Display, Formatter, self};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clue {
    pub entered_by: Player,
    pub text: String
}

impl Display for Clue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "```\nEntered by: {}\n\n{}\n```", self.entered_by.name, self.text)
    }
}
