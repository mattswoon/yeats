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

    pub fn add_clue(self, c: Clue) -> Bowl {
        Bowl {
            unsolved: self.unsolved
                .into_iter()
                .chain(vec![c].into_iter())
                .collect(),
            ..self}
    }
}
