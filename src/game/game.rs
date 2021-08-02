use std::{
    iter::Cycle,
    vec::IntoIter
};
use rand::{
    thread_rng,
    seq::SliceRandom,
};
use serenity::{
    prelude::TypeMapKey,
    model::{
        user::User,
        channel::GuildChannel,
    },
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
    pub num_rounds: i64,
    pub main_channel: Option<GuildChannel>
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
            num_rounds: 3,
            main_channel: None
        }
    }

    pub fn reset(&mut self) -> Option<String> {
        *self = Game::new();
        None
    }

    pub fn status(&self) -> Option<String> {
        match &self.state {
            GameState::PreGame => {
                Some(format!("Game is yet to start, feel free to join or add more clues.\n\tPlayers:\t{}\n\tClues added:\t{}", 
                        self.players
                            .iter()
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                            .join("\n\t\t"), 
                        self.bowl.status()))
            },
            GameState::Round(round) => {
                let turn_status = round.current_turn
                    .as_ref()
                    .map(Turn::status)
                    .unwrap_or("".to_string());
                Some(format!("We're currently playing round {}. {}", 
                        &round.round_number, 
                        turn_status)
                    .trim()
                    .to_string())
            },
            _ => Some(format!("I dunno??")),
        }
    }

    pub fn set_admin(&mut self, user: User) {
        self.admin = Some(user);
    }

    pub fn add_players(&mut self, p: &mut Vec<Player>) -> Result<Option<String>, GameError> {
        match self.state {
            GameState::PreGame => {
                let reply = format!("Added players {}", p.iter()
                                    .cloned()
                                    .map(|p| p.name)
                                    .collect::<Vec<_>>()
                                    .join(", "));
                (*self).players.append(p);
                Ok(Some(reply))
            },
            _ => Err(GameError::AlreadyStarted)
        }
    }

    pub fn add_player(&mut self, p: Player) -> Result<Option<String>, GameError> {
        match self.state {
            GameState::PreGame => {
                let reply = format!("{} joined the game", &p.name);
                (*self).players.push(p);
                Ok(Some(reply))
            },
            _ => Err(GameError::AlreadyStarted)
        }
    }

    pub fn list_players(&self) -> Option<String> {
        Some(self.players
             .iter()
             .map(|p| p.name.clone())
             .collect::<Vec<_>>()
             .join("\n"))
    }

    pub fn add_clue(&mut self, c: Clue) -> Result<Option<String>, GameError> {
        match self.state {
            GameState::PreGame => {
                self.bowl.add_clue(c);
                Ok(None)
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

    pub fn start_game(&mut self, channel: GuildChannel) -> Result<Option<String>, GameError> {
        match &self.state {
            GameState::PreGame => {
                self.state = GameState::Round(Round::new(1, &self.players));
                self.main_channel = Some(channel);
                Ok(Some("Welcome to the game".to_string()))
            },
            _ => Err(GameError::AlreadyStarted)
        }
    }

    pub fn prepare_turn(&mut self) -> Result<Option<String>, GameError> {
        match &self.state {
            GameState::Round(r) => r.clone().prepare_turn().map(GameState::Round),
            GameState::PreGame => Err(GameError::CantDoThat),
            GameState::End => Err(GameError::CantDoThat)
        }.map(|new_state| self.state = new_state)
        .and_then(|_| self.ready_turn_message().map(Some))
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

    pub fn current_performer(&self) -> Option<Player> {
        match &self.state {
            GameState::Round(Round { 
                round_number: _,
                turn_queue: _,
                current_turn: Some(Turn { performer, guesser: _, state: _})
            }) => Some(performer.clone()),
            _ => None
        }
    }

    pub fn draw_clue(&mut self, by: &Player) -> Result<DrawClue, GameError> {
        match &self.state {
            GameState::Round(round) => {
                match &round.current_turn {
                    Some(Turn { performer, guesser, state: TurnState::Guessing }) =>
                        if performer == by {
                            let clue = self.bowl.draw_clue();
                            Ok(DrawClue {
                                clue: clue,
                                performer: performer.clone(),
                                guesser: guesser.clone(),
                            })
                        } else {
                            Err(GameError::PlayerNotAllowedToDrawAClue(performer.clone()))
                        },
                    Some(Turn { performer, guesser, state: TurnState::Ready }) => 
                        Err(GameError::BadTurnState(Turn { 
                            performer: performer.clone(), 
                            guesser: guesser.clone(), 
                            state: TurnState::Ready})),
                    Some(Turn { performer, guesser, state: TurnState::Ended }) => 
                        Err(GameError::BadTurnState(Turn { 
                            performer: performer.clone(), 
                            guesser: guesser.clone(), 
                            state: TurnState::Ended})),
                    None => Err(GameError::NoTurnsQueued),
                }
            },
            GameState::PreGame => Err(GameError::CantDoThat),
            GameState::End => Err(GameError::CantDoThat),
        }
    }
}

pub enum GameState {
    PreGame,
    Round(Round),
    End
}

#[derive(Debug, Clone)]
pub struct DrawClue {
    pub clue: Option<Clue>,
    pub performer: Player,
    pub guesser: Player,
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
