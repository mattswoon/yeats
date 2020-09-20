use std::{
    iter::Cycle,
    vec::IntoIter
};
use serenity::prelude::TypeMapKey;
use crate::game::{
    game_error::GameError,
    player::Player,
    turn::{TurnState, Turn},
    bowl::Bowl,
    clue::Clue,
};

pub struct Game {
    pub players: Vec<Player>,
    pub bowl: Bowl,
    pub state: GameState,
    pub turn_queue: Cycle<IntoIter<Turn>>
}

impl TypeMapKey for Game {
    type Value = Game;
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: vec![],
            bowl: Bowl::new(),
            state: GameState::PreGame,
            turn_queue: vec![].into_iter().cycle()
        }
    }

    pub fn add_players(&mut self, p: &mut Vec<Player>) -> Result<(), GameError> {
        match self.state {
            GameState::PreGame => {
                (*self).players.append(p);
                Ok(())
            },
            _ => Err(GameError::AlreadyStarted)
        }
    }

//    pub fn add_clue(self, c: Clue) -> Result<Game, GameError> {
//        match self.state {
//            GameState::PreGame => {
//                let bowl = self.bowl.add_clue(c);
//                Ok(Game {
//                    bowl: bowl,
//                    ..self})
//            },
//            _ => Err(GameError::AlreadyStarted)
//        }
//    }
//
//    pub fn start_game(self) -> Result<Game, GameError> {
//        match self.state {
//            GameState::PreGame => {
//                let turn_queue = self.players
//                    .clone()
//                    .into_iter()
//                    .cycle()
//                    .zip(self.players
//                         .clone()
//                         .into_iter()
//                         .cycle()
//                         .skip(1))
//                    .map(|(p1, p2)| Turn::new(p1, p2))
//                    .collect::<Vec<_>>()
//                    .into_iter()
//                    .cycle();
//                let turn = turn_queue.next();
//                Ok(Game {
//                    turn_queue: turn_queue,
//                    state: GameState::Playing,
//                    ..self })
//            },
//            _ => Err(GameError::AlreadyStarted)
//        }
//    }

//    pub fn ready_turn(self) -> Result<Game, GameError> {
//        match self.state {
//            GameState
//    }
}

pub enum GameState {
    PreGame,
    Turn(Turn),
    Playing,
}
