use crate::game::{
    player::Player,
};

#[derive(Clone)]
pub struct Clue {
    entered_by: Player,
    text: String
}
