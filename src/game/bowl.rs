use rand::{
    thread_rng,
    seq::SliceRandom,
};
use std::collections::HashMap;
use crate::game::{
    clue::Clue,
};

pub struct Bowl {
    unsolved: Vec<Clue>,
    solved: Vec<Clue>,
    showing: Option<Clue>,
}

impl Bowl {
    pub fn new() -> Bowl {
        Bowl {
            unsolved: vec![],
            solved: vec![],
            showing: None,
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
        let clue = self.unsolved.pop();
        self.showing = clue.clone();
        clue
    }

    pub fn put_back(&mut self) {
        if let Some(c) = &self.showing {
            self.unsolved.push(c.clone());
            self.showing = None;
            self.shuffle();
        }
    }

    pub fn solve_showing_clue(&mut self) {
        if let Some(c) = &self.showing {
            self.solved.push(c.clone());
            self.showing = None;
        }
    }

    pub fn status(&self) -> String {
        self.unsolved
            .iter()
            .chain(self.solved.iter())
            .fold(HashMap::new(), |acc, item| {
                let mut acc = acc;
                match acc.get_mut(&item.entered_by.name) {
                    Some(v) => { *v += 1; ()},
                    None => { acc.insert(item.entered_by.name.clone(), 1); ()}
                };
                acc
            })
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n\t\t")
    }
}
