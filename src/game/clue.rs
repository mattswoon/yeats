use crate::game::{
    player::Player,
};

#[derive(Clone)]
pub struct Clue {
    pub entered_by: Player,
    pub text: String
}
