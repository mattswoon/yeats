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
use crate::{
    error::Error,
    game::{
        game_error::GameError,
        player::Player,
        turn::{TurnState, Turn, TurnSummary},
        bowl::Bowl,
        clue::Clue,
    },
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

    pub fn reset(&mut self) {
        *self = Game::new();
    }

    pub fn status(&self) -> String {
        match &self.state {
            GameState::PreGame => {
                format!("Game is yet to start, feel free to join or add more clues.\n\tPlayers:\t{}\n\tClues added:\t{}", 
                        self.players
                            .iter()
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                            .join("\n\t\t"), 
                        self.bowl.status())
            },
            GameState::Round(round) => {
                let turn_status = round.current_turn
                    .as_ref()
                    .map(Turn::status)
                    .unwrap_or("".to_string());
                format!("We're currently playing round {}. {}", 
                        &round.round_number, 
                        turn_status)
                    .trim()
                    .to_string()
            },
            GameState::End => format!("Game as finished"),
            _ => format!("I dunno??"),
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

    pub fn add_player(&mut self, p: Player) -> Result<(), Error> {
        match self.state {
            GameState::PreGame => {
                (*self).players.push(p);
                Ok(())
            },
            _ => Err(Error::GameAlreadyStarted)
        }
    }

    pub fn list_players(&self) -> Option<String> {
        Some(self.players
             .iter()
             .map(|p| p.name.clone())
             .collect::<Vec<_>>()
             .join("\n"))
    }

    pub fn add_clue(&mut self, c: &Clue) -> Result<(), Error> {
        match self.state {
            GameState::PreGame => {
                self.bowl.add_clue(c);
                Ok(())
            },
            _ => Err(Error::GameAlreadyStarted)
        }
    }

    pub fn advance_game(&mut self) -> Result<(), Error> {
        let new_state = match &self.state {
            GameState::PreGame => {
                if self.main_channel.is_none() {
                    Err(Error::NoChannel)
                } else {
                    Ok(GameState::Round(
                        Round::new(1, &self.players)
                    ))
                }
            },
            GameState::Round(r) => {
                if self.bowl.num_unsolved() > 0 {
                    Err(Error::BowlNotEmpty)
                } else {
                    if r.round_number < self.num_rounds {
                        self.bowl = self.bowl.clone().refill();
                        Ok(GameState::Round(
                            Round::new(r.round_number + 1, &self.players)
                        ))
                    } else {
                        Ok(GameState::End)
                    }
                }
            },
            GameState::End => Err(Error::GameFinished)
        }?;
        self.state = new_state;
        Ok(())
    }

    pub fn start_game(&mut self, channel: GuildChannel) -> Result<(), Error> {
        match &self.state {
            GameState::PreGame => {
                self.state = GameState::Round(Round::new(1, &self.players));
                self.main_channel = Some(channel);
                Ok(())
            },
            _ => Err(Error::GameAlreadyStarted)
        }
    }

    pub fn prepare_turn(&mut self) -> Result<Turn, Error> {
        if self.bowl.num_unsolved() == 0 {
            return Err(Error::EmptyBowl);
        }
        match &self.state {
            GameState::Round(r) => r.clone().prepare_turn().map(GameState::Round),
            GameState::PreGame => Err(Error::GameNotStartedYet),
            GameState::End => Err(Error::GameFinished)
        }.and_then(|new_state| {
            self.state = new_state.clone();
            match new_state {
                GameState::Round(r) => r.current_turn.ok_or(Error::NoTurnsQueued),
                GameState::PreGame => Err(Error::GameNotStartedYet),
                GameState::End => Err(Error::GameFinished),
            }
        })
    }

    pub fn ready_turn_message(&self) -> Result<String, GameError> {
        match &self.state {
            GameState::Round(r) => r.ready_turn_message(),
            GameState::PreGame => Err(GameError::CantDoThat),
            GameState::End => Err(GameError::CantDoThat),
        }
    }

    pub fn start_turn(&mut self) -> Result<(Turn, i64), Error> {
        let (new_state, turn, round_number) = match &self.state {
            GameState::Round(r) => r.clone()
                .start_turn()
                .map(|(new_r, t)| (GameState::Round(new_r), t, r.round_number)),
            GameState::PreGame => Err(Error::GameNotStartedYet),
            GameState::End => Err(Error::GameFinished)
        }?;
        self.state = new_state;
        Ok((turn, round_number))
    }
    
    pub fn end_turn(&mut self, p: &Player, g: &Player, round_number: i64) -> Result<(), Error> {
        let new_state = match &self.state {
            GameState::Round(r) => r.clone().end_turn(p, g, round_number).map(GameState::Round),
            GameState::PreGame => Err(Error::GameNotStartedYet),
            GameState::End => Err(Error::GameFinished)
        }?;
        self.bowl.put_back();
        self.state = new_state;
        Ok(())
    }

    pub fn turn_summary(&self) -> Result<TurnSummary, Error> {
        match &self.state {
            GameState::Round(r) => { // HERE
                match r.current_turn.clone().ok_or(Error::NoTurnsQueued)?.state {
                    TurnState::Ready => Err(Error::CurrentTurnNotYetStarted),
                    TurnState::Guessing(summ) => Ok(summ),
                    TurnState::Ended(summ) => Ok(summ),
                }
            },
            GameState::PreGame => Err(Error::GameNotStartedYet),
            GameState::End => Err(Error::GameFinished),
        }
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

    pub fn current_round_number(&self) -> Option<i64> {
        match &self.state {
            GameState::Round(r) => Some(r.round_number),
            _ => None
        }
    }

    pub fn draw_clue(&mut self, by: &Player) -> Result<DrawClue, Error> {
        let (draw_clue, state) = match &self.state {
            GameState::Round(round) => {
                match &round.current_turn {
                    Some(Turn { performer, guesser, state: TurnState::Guessing(summ) }) =>
                        if performer == by {
                            let summ = self.bowl.showing()
                                .map(|showing| summ.clone().with_clue(showing))
                                .unwrap_or(summ.clone());
                            
                            self.bowl.solve_showing_clue();
                            let clue = self.bowl.draw_clue();
                            Ok((DrawClue {
                                clue: clue,
                                performer: performer.clone(),
                                guesser: guesser.clone(),
                            }, GameState::Round(round.clone().with_current_turn(round.current_turn.clone().map(|t| t.with_state(TurnState::Guessing(summ)))))))
                        } else {
                            Err(Error::PlayerNotAllowedToDrawAClue)
                        },
                    Some(Turn { performer, guesser, state: TurnState::Ready }) => 
                        Err(Error::CurrentTurnNotYetStarted),
                    Some(Turn { performer, guesser, state: TurnState::Ended(_) }) => 
                        Err(Error::CurrentTurnHasEnded),
                    None => Err(Error::NoTurnsQueued),
                }
            },
            GameState::PreGame => Err(Error::GameNotStartedYet),
            GameState::End => Err(Error::GameFinished),
        }?;
        self.state = state;
        Ok(draw_clue)
    }
}

#[derive(Debug, Clone)]
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
            .zip(players.iter().cycle().skip(1))
            .map(|(p1, p2)| Turn::new(p1.clone(), p2.clone()))
            .collect();
        Round { round_number, turn_queue, current_turn: None }
    }

    pub fn with_current_turn(self, current_turn: Option<Turn>) -> Round {
        Round { current_turn, ..self }
    }

    pub fn prepare_turn(self) -> Result<Round, Error> {
        match self.current_turn {
            None => {
                let mut turn_queue = self.turn_queue;
                let current_turn = turn_queue.pop()
                    .ok_or(Error::EmptyTurnQueue)?;
                Ok(Round { 
                    round_number: self.round_number,
                    turn_queue,
                    current_turn: Some(current_turn)
                })
            },
            Some(Turn { performer, guesser, state: TurnState::Ended(_) }) => {
                let ended_turn = Turn::new(performer.clone(), guesser.clone());
                let mut turn_queue = self.turn_queue;
                turn_queue.insert(0, ended_turn);
                let current_turn = turn_queue.pop()
                    .ok_or(Error::EmptyTurnQueue)?;
                Ok(Round {
                    round_number: self.round_number,
                    turn_queue,
                    current_turn: Some(current_turn)
                })
            },
            Some(Turn { state: TurnState::Ready, .. }) => {
                Err(Error::CurrentTurnNotYetFinished)
            },
            Some(Turn { state: TurnState::Guessing(_), .. }) => {
                Err(Error::CurrentTurnNotYetFinished)
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
                TurnState::Guessing(_) => Err(GameError::BadTurnState(t.clone())),
                TurnState::Ended(_) => Err(GameError::BadTurnState(t.clone())),
            },
            None => Err(GameError::NoTurnsQueued)
        }
    }

    pub fn start_turn(self) -> Result<(Round, Turn), Error> {
        match self.current_turn {
            Some(t) => match t.state {
                TurnState::Ready => Ok((Round {
                    round_number: self.round_number,
                    turn_queue: self.turn_queue,
                    current_turn: Some(t.clone().as_guessing())
                }, t.as_guessing())),
                TurnState::Guessing(_) => Err(Error::CurrentTurnNotYetFinished),
                TurnState::Ended(_) => Err(Error::CurrentTurnHasEnded),
            },
            None => Err(Error::NoTurnsQueued),
        }
    }

    pub fn end_turn(self, p: &Player, g: &Player, round_number: i64) -> Result<Round, Error> {
        match self.current_turn {
            Some(t) => {
                if (p == &t.performer) & (g == &t.guesser) & (self.round_number == round_number) {
                    match t.state {
                        TurnState::Guessing(_) => Ok(Round {
                            round_number: self.round_number,
                            turn_queue: self.turn_queue,
                            current_turn: Some(t.as_ended())
                        }),
                        TurnState::Ready => Err(Error::CurrentTurnNotYetStarted),
                        TurnState::Ended(_) => Err(Error::CurrentTurnHasEnded),
                    }
                } else {
                    Err(Error::TurnDoesntMatchPlayers)
                }
            },
            None => Err(Error::NoTurnsQueued)
        }
    }
}
