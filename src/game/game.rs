use std::{
    iter::Cycle,
    vec::IntoIter
};
use tokio::time::{Duration, sleep};
use rand::{
    thread_rng,
    seq::SliceRandom,
};
use serenity::{
    prelude::TypeMapKey,
    model::user::User,
};
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
    pub admin: Option<User>,
    pub num_rounds: i64
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
            admin: None,
            num_rounds: 3
        }
    }

    pub fn set_admin(&mut self, user: User) {
        self.admin = Some(user);
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

    pub fn add_player(&mut self, p: Player) -> Result<(), GameError> {
        match self.state {
            GameState::PreGame => {
                (*self).players.push(p);
                Ok(())
            },
            _ => Err(GameError::AlreadyStarted)
        }
    }

    pub fn add_clue(&mut self, c: Clue) -> Result<(), GameError> {
        match self.state {
            GameState::PreGame => {
                self.bowl.add_clue(c);
                Ok(())
            },
            _ => Err(GameError::AlreadyStarted)
        }
    }

    pub fn advance_game(&mut self) -> Result<(), GameError> {
        match &self.state {
            GameState::PreGame => {
                self.state = GameState::Round(
                    Round::new(1, &self.players)
                )
            },
            GameState::Round(r) => {
                if r.round_number < self.num_rounds {
                    self.state = GameState::Round(
                        Round::new(r.round_number + 1, &self.players)
                    )
                } else {
                    self.state = GameState::End
                }
            },
            GameState::End => ()
        }
        Ok(())
    }

    pub fn start_game(&mut self) -> Result<(), GameError> {
        match &self.state {
            GameState::PreGame => {
                self.state = GameState::Round(Round::new(1, &self.players));
                Ok(())
            },
            _ => Err(GameError::AlreadyStarted)
        }
    }

    pub fn start_game_message(&self) -> String {
        "Welcome to the game".to_string()
    }
}

pub enum GameState {
    PreGame,
    Round(Round),
    End
}

#[derive(Debug, Clone)]
pub struct Round {
    pub round_number: i64,
    pub turn_queue: Vec<Turn>,
    pub current_turn: Option<Turn>
}

impl Round {
    pub fn new(round_number: i64, players: &Vec<Player>) -> Round {
        let mut players = players.clone();
        let mut rng = thread_rng();
        players.shuffle(&mut rng);
        let turn_queue = players.iter()
            .zip(players.iter().cycle())
            .map(|(p1, p2)| Turn::new(p1.clone(), p2.clone()))
            .collect();
        Round { round_number, turn_queue, current_turn: None }
    }

    pub fn prepare_turn(&mut self) -> Result<(), GameError> {
        match &self.current_turn {
            None => {
                let turn = self.turn_queue.pop();
                self.current_turn = turn;
                Ok(())
            },
            Some(Turn { performer, guesser, state: TurnState::Ended }) => {
                let ended_turn = Turn::new(performer.clone(), guesser.clone());
                self.turn_queue.insert(0, ended_turn);
                let turn = self.turn_queue.pop();
                self.current_turn = turn;
                Ok(())
            },
            Some(t) => {
                Err(GameError::BadTurnState(t.clone()))
            }
        }
    }

    pub async fn start_turn(&mut self) -> Result<(), GameError> {
        match &self.current_turn {
            Some(t) => match t.state {
                TurnState::Ready => {
                    let p = t.performer.clone();
                    let g = t.guesser.clone();
                    self.current_turn = self.current_turn.as_ref().map(Turn::as_guessing);
                    sleep(Duration::new(60, 0)).await;
                    self.end_turn(&p, &g)?;
                    Ok(())
                },
                _ => Err(GameError::BadTurnState(t.clone())),
            }
            None => Err(GameError::NoTurnsQueued)
        }
    }

    pub fn end_turn(&mut self, p: &Player, g: &Player) -> Result<(), GameError> {
        match &self.current_turn {
            Some(Turn { performer, guesser, state }) => {
                if (performer == p) & (guesser == g) {
                    self.current_turn = self.current_turn
                        .as_ref()
                        .map(Turn::as_ended);
                    Ok(())
                } else {
                    Err(GameError::TurnDoesntMatchPlayers {
                        turn: Turn { 
                            performer: performer.clone(), 
                            guesser: guesser.clone(), 
                            state: state.clone() 
                        },
                        performer: p.clone(),
                        guesser: g.clone() })
                }
            },
            None => Err(GameError::NoTurnsQueued)
        }
    }
}
