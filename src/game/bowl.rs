use crate::game::{
    clue::Clue,
};

pub struct Bowl {
    unsolved: Vec<Clue>,
    solved: Vec<Clue>
}

impl Bowl {
    pub fn new() -> Bowl {
        Bowl {
            unsolved: vec![],
            solved: vec![]
        }
    }

    pub fn add_clue(&mut self, c: Clue) -> () {
        self.unsolved.append(&mut vec![c]);
    }
}
