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
    utils::MessageBuilder,
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

    pub fn prepare_turn(&mut self) -> Result<(), GameError> {
        let new_state = match &self.state {
            GameState::Round(r) => r.clone().prepare_turn().map(GameState::Round),
            GameState::PreGame => Err(GameError::CantDoThat),
            GameState::End => Err(GameError::CantDoThat)
        }?;
        self.state = new_state;
        Ok(())
    }

    pub fn ready_turn_message(&self) -> Result<String, GameError> {
        match &self.state {
            GameState::Round(r) => r.ready_turn_message(),
            GameState::PreGame => Err(GameError::CantDoThat),
            GameState::End => Err(GameError::CantDoThat),
        }
    }

    pub fn start_turn(&mut self) -> Result<Turn, GameError> {
        let (new_state, turn) = match &self.state {
            GameState::Round(r) => r.clone()
                .start_turn()
                .map(|(new_r, t)| (GameState::Round(new_r), t)),
            GameState::PreGame => Err(GameError::CantDoThat),
            GameState::End => Err(GameError::CantDoThat)
        }?;
        self.state = new_state;
        Ok(turn)
    }
    
    pub fn end_turn(&mut self, p: &Player, g: &Player) -> Result<(), GameError> {
        let new_state = match &self.state {
            GameState::Round(r) => r.clone().end_turn(p, g).map(GameState::Round),
            GameState::PreGame => Err(GameError::CantDoThat),
            GameState::End => Err(GameError::CantDoThat)
        }?;
        self.state = new_state;
        Ok(())
    }

    pub async fn run_turn(&mut self) -> Result<(), GameError> {
        let Turn { performer, guesser, state: _} = self.start_turn()?;
        sleep(Duration::from_secs(60)).await;
        self.end_turn(&performer, &guesser)
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
            .zip(players.iter().skip(1).cycle())
            .map(|(p1, p2)| Turn::new(p1.clone(), p2.clone()))
            .collect();
        Round { round_number, turn_queue, current_turn: None }
    }

    pub fn prepare_turn(self) -> Result<Round, GameError> {
        match self.current_turn {
            None => {
                let mut turn_queue = self.turn_queue;
                let current_turn = turn_queue.pop();
                Ok(Round { 
                    round_number: self.round_number,
                    turn_queue,
                    current_turn
                })
            },
            Some(Turn { performer, guesser, state: TurnState::Ended }) => {
                let ended_turn = Turn::new(performer.clone(), guesser.clone());
                let mut turn_queue = self.turn_queue;
                turn_queue.insert(0, ended_turn);
                let current_turn = turn_queue.pop();
                Ok(Round {
                    round_number: self.round_number,
                    turn_queue,
                    current_turn
                })
            },
            Some(t) => {
                Err(GameError::BadTurnState(t.clone()))
            }
        }
    }

    pub fn ready_turn_message(&self) -> Result<String, GameError> {
        match &self.current_turn {
            Some(t) => match t.state {
                TurnState::Ready => Ok(
                    MessageBuilder::new()
                        .push("Get ready ")
                        .mention(&t.performer.user)
                        .push(", you'll be performing for ")
                        .mention(&t.guesser.user)
                        .push("!!")
                        .build()
                ),
                TurnState::Guessing => Err(GameError::BadTurnState(t.clone())),
                TurnState::Ended => Err(GameError::BadTurnState(t.clone())),
            },
            None => Err(GameError::NoTurnsQueued)
        }
    }

    pub fn start_turn(self) -> Result<(Round, Turn), GameError> {
        match self.current_turn {
            Some(t) => match t.state {
                TurnState::Ready => Ok((Round {
                    round_number: self.round_number,
                    turn_queue: self.turn_queue,
                    current_turn: Some(t.clone().as_guessing())
                }, t.as_guessing())),
                TurnState::Guessing => Err(GameError::BadTurnState(t)),
                TurnState::Ended => Err(GameError::BadTurnState(t)),
            },
            None => Err(GameError::NoTurnsQueued),
        }
    }

    pub fn end_turn(self, p: &Player, g: &Player) -> Result<Round, GameError> {
        match self.current_turn {
            Some(t) => {
                if (p == &t.performer) & (g == &t.guesser) {
                    match t.state {
                        TurnState::Guessing => Ok(Round {
                            round_number: self.round_number,
                            turn_queue: self.turn_queue,
                            current_turn: Some(t.as_ended())
                        }),
                        TurnState::Ready => Err(GameError::BadTurnState(t)),
                        TurnState::Ended => Err(GameError::BadTurnState(t))
                    }
                } else {
                    Err(GameError::TurnDoesntMatchPlayers {
                        turn: t,
                        performer: p.clone(),
                        guesser: g.clone()
                    })
                }
            },
            None => Err(GameError::NoTurnsQueued)
        }
    }
}
