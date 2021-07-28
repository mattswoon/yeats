use rand::{
    thread_rng,
    seq::SliceRandom,
};
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

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.unsolved.shuffle(&mut rng);
    }

    pub fn draw_clue(&mut self) -> Option<Clue> {
        self.unsolved.pop()
    }

    pub fn put_back(&mut self, c: Clue) {
        self.unsolved.push(c);
        self.shuffle();
    }

    pub fn mark_solved(&mut self, c: Clue) {
        self.solved.push(c);
    }
}
