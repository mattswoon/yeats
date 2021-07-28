use crate::game::{
    player::Player,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clue {
    pub entered_by: Player,
    pub text: String
}
