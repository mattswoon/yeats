use rand::{
    thread_rng,
    seq::SliceRandom,
};
use std::collections::HashMap;
use crate::game::{
    clue::Clue,
    player::Player,
};

#[derive(Debug, Clone)]
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

    pub fn showing(&self) -> Option<Clue> {
        self.showing.clone()
    }

    pub fn add_clue(&mut self, c: &Clue) {
        self.unsolved.append(&mut vec![c.clone()]);
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.unsolved.shuffle(&mut rng);
    }

    pub fn draw_clue(&mut self, not_entered_by: &Player) -> Option<Clue> {
        let (clue, pool) = CluePool::new(self.unsolved.clone()).draw(|c| &c.entered_by == not_entered_by);
        self.unsolved = pool.to_vec();

        log::debug!("{:?} being shown", &clue);
        self.showing = clue.clone();
        clue
    }

    pub fn put_back(&mut self) {
        if let Some(c) = &self.showing {
            log::debug!("{} marked as unsolved", &c);
            self.unsolved.push(c.clone());
            self.showing = None;
            self.shuffle();
        }
    }

    pub fn solve_showing_clue(&mut self) {
        if let Some(c) = &self.showing {
            log::debug!("{} marked as solved", &c);
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

    pub fn num_unsolved(&self) -> usize {
        self.unsolved.len()
    }

    pub fn refill(self) -> Bowl {
        let unsolved = self.unsolved
            .into_iter()
            .chain(self.solved.into_iter())
            .collect();
        Bowl {
            unsolved,
            solved: vec![],
            showing: None
        }
    }
}

struct CluePool<C> {
    pool: Vec<C>,
    rejected: Vec<C>
}

impl<C: Clone> CluePool<C> {
    fn new(pool: Vec<C>) -> CluePool<C> {
        CluePool { pool, rejected: vec![] }
    }

    fn to_vec(self) -> Vec<C> {
        let CluePool { pool, rejected } = self;
        [pool, rejected].concat().to_vec()
    }

    fn draw<F>(self, reject_if: F) -> (Option<C>, CluePool<C>) 
    where
        F: Fn(&C) -> bool
    {
        let CluePool { pool, rejected } = self;
        match pool.split_first() {
            Some((head, tail)) => {
                let pool = tail.to_vec();
                if reject_if(head) {
                    let rejected = [rejected, vec![head.clone()]].concat().to_vec();
                    return CluePool { pool, rejected }.draw(reject_if)
                } else {
                    return (Some(head.clone()), CluePool { pool, rejected })
                }
            },
            None => {
                let pool = vec![];
                let mut rejected = rejected;
                let clue = rejected.pop();
                (clue, CluePool { pool, rejected })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clue_pool() {
        let clues = [
            ("player", "test clue"),
            ("player", "a different clue"),
            ("a different player", "even another clue")
        ].to_vec();
        let (clue, pool) = CluePool::new(clues).draw(|c| c.0 == "player");
        assert_eq!(clue, Some(("a different player", "even another clue")));
        assert_eq!(pool.to_vec(), [("player", "test clue"), ("player", "a different clue")].to_vec());
    }
    
    #[test]
    fn test_clue_pool_no_good_options() {
        let clues = [
            ("player", "test clue"),
            ("player", "a different clue"),
        ].to_vec();
        let (clue, pool) = CluePool::new(clues).draw(|c| c.0 == "player");
        assert_eq!(clue, Some(("player", "a different clue")));
        assert_eq!(pool.to_vec(), [("player", "test clue")].to_vec());
    }
}
