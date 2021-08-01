use tokio::time::{Duration, sleep};
use serenity::{
    prelude::*,
    model::prelude::*,
    client::ClientBuilder,
    framework::standard::{
        Args,
        StandardFramework,
        CommandResult,
        macros::{command, group},
    },
};
use itertools::Itertools;
use yeats::game::{
    game::Game,
    player::Player,
    game_error::GameError,
    clue::Clue,
    turn::Turn,
};

#[derive(Debug)]
enum Error {
    NoGame,
    GameError(GameError),
    Serenity(serenity::prelude::SerenityError)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoGame => 
                write!(f, "No Game object in context"),
            Error::GameError(ge) => write!(f, "{}", ge),
            Error::Serenity(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}


enum Response {
    Reply(String),
    ThumbsUp,
}

struct ValuedResponse<T> {
    response: Option<Response>,
    value: T
}

impl<T> ValuedResponse<T> {
    fn from(value: T) -> Self {
        ValuedResponse { response: None, value }
    }

    fn with_reply(self, reply: String) -> Self {
        ValuedResponse {
            response: Some(Response::Reply(reply)),
            value: self.value
        }
    }

    fn with_thumbsup(self) -> Self {
        ValuedResponse {
            response: Some(Response::ThumbsUp),
            value: self.value
        }
    }
}

async fn respond_and_return<T>(ctx: &Context, msg: &Message, result: Result<ValuedResponse<T>, Error>) -> Result<T, Error> {
    match result {
        Ok(ValuedResponse { response: Some(Response::Reply(reply)), value }) => 
            msg.reply(ctx, reply)
                .await
                .map(|_| value)
                .map_err(Error::Serenity),
        Ok(ValuedResponse { response: Some(Response::ThumbsUp), value }) =>
            msg.react(ctx, '👍')
                .await
                .map(|_| value)
                .map_err(Error::Serenity),
        Ok(ValuedResponse { response: None, value }) => Ok(value),
        Err(Error::NoGame) => {
            log::warn!("No game data");
            msg.react(ctx, '❗').await.map_err(Error::Serenity)?;
            msg.reply(ctx, "Things have gone very bad, ask Tom").await.map_err(Error::Serenity)?;
            Err(Error::NoGame)
        },
        Err(Error::GameError(e)) => {
            log::info!("GameError: {}", &e);
            msg.react(ctx, '🚫').await.map_err(Error::Serenity)?;
            msg.reply(ctx, &e).await.map_err(Error::Serenity)?;
            Err(Error::GameError(e))
        },
        Err(Error::Serenity(e)) => {
            log::warn!("{}", &e);
            msg.react(ctx, '❗').await.map_err(Error::Serenity)?;
            msg.reply(ctx, "Things have gone very bad, ask Tom").await.map_err(Error::Serenity)?;
            Err(Error::Serenity(e))
        }
    }
}

async fn respond(ctx: &Context, msg: &Message, result: Result<Option<String>, Error>) -> CommandResult {
    match result {
        Ok(Some(s)) => { msg.reply(ctx, s).await?; },
        Err(e) => match e {
            Error::NoGame => {
                log::warn!("No game data");
                msg.react(ctx, '❗').await?;
                msg.reply(ctx, "Things have gone very bad, ask Tom").await?;
            },
            Error::GameError(ge) => {
                log::info!("GameError: {}", ge);
                msg.react(ctx, '🚫').await?;
                msg.reply(ctx, ge).await?;
            }
            Error::Serenity(se) => {
                log::warn!("{}", &se);
                msg.react(ctx, '❗').await.map_err(Error::Serenity)?;
                msg.reply(ctx, "Things have gone very bad, ask Tom").await?;
            }
        },
        Ok(None) => { msg.react(ctx, '👍').await?; }
    };
    Ok(())
}

#[command]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is checking status", msg.author.name);
    respond(
        ctx, 
        msg,
        ctx.data
            .read()
            .await
            .get::<Game>()
            .ok_or(Error::NoGame)
            .map(Game::status)).await
}

#[command]
async fn reset(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is resetting the game", msg.author.name);
    respond(
        ctx,
        msg,
        ctx.data.write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .map(Game::reset)).await
}

#[command]
#[aliases("add-players")]
async fn add_players(ctx: &Context, msg: &Message) -> CommandResult {
    let mut players: Vec<Player> = msg.mentions
        .iter()
        .map(|u| u.into())
        .collect();
    log::info!("{} is adding players {}", msg.author.name, &players.iter().join(", "));
    respond(
        ctx,
        msg,
        ctx.data.write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.add_players(&mut players).map_err(Error::GameError))).await
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is joining the game", msg.author.name);
    let player: Player = (&msg.author).into();
    respond(
        ctx,
        msg,
        ctx.data.write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.add_player(player).map_err(Error::GameError))).await
}

#[command]
#[aliases("list-players")]
async fn list_players(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is listing players", msg.author.name);
    respond(
        ctx,
        msg,
        ctx.data.read()
            .await
            .get::<Game>()
            .ok_or(Error::NoGame)
            .map(Game::list_players)).await
}

#[command]
#[aliases("add-clue")]
async fn add_clue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.is_private() {
        let text = args.rest().into();
        let entered_by: Player = (&msg.author).into();
        log::info!("Adding clue from {}", &entered_by);
        respond(
            ctx,
            msg,
            ctx.data
                .write()
                .await
                .get_mut::<Game>()
                .ok_or(Error::NoGame)
                .and_then(|g| g.add_clue(Clue { entered_by, text })
                          .map_err(Error::GameError))).await
    } else {
        respond(ctx, msg, Ok(Some("Umm... you're supposed to dm that to me".to_string()))).await
    }
}

#[command]
#[aliases("debug-mode")]
async fn debug_mode(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.is_private() {
        if args.rest() == "motherlode" {
            log::info!("Setting {} as game admin", &msg.author);
            ctx.data
                .write()
                .await
                .get_mut::<Game>()
                .map(|g| g.set_admin(msg.author.clone()))
                .ok_or(Error::NoGame)?;
            msg.reply(ctx, "Shh, it's our little secret").await?;
        } else {
            log::info!("User {} tried to set themselves as admin but got the password wrong", 
                       &msg.author);
        }
    } else {
        log::info!("User {} tried to set themselves as admin but not in a private channel",
                   &msg.author);
    }
    Ok(())
}

#[command]
#[aliases("start-game")]
async fn start_game(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Starting game");
    respond(
        ctx,
        msg,
        ctx.data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.start_game().map_err(Error::GameError))).await
}

#[command]
#[aliases("next-turn")]
async fn next_turn(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Queueing next turn");
    respond(
        ctx,
        msg,
        ctx.data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.prepare_turn().map_err(Error::GameError))).await
}

#[command]
#[aliases("start-turn")]
async fn start_turn(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Starting turn");
    let Turn { performer, guesser, state: _ } = 
        respond_and_return(
            ctx,
            msg,
            ctx.data
                .write()
                .await
                .get_mut::<Game>()
                .ok_or(Error::NoGame)
                .and_then(|g| g.start_turn().map_err(Error::GameError))
                .map(|t| ValuedResponse::from(t.clone())
                     .with_reply(format!("GO! {} you've got 60 seconds to perform as many clues as possible to {}", 
                                         &t.performer,
                                         &t.guesser)))).await?;
    log::info!("Starting timer for {} -> {}", &performer, &guesser);
    sleep(Duration::from_secs(60)).await;
    log::info!("Ending turn {} -> {}", &performer, &guesser);
    ctx.data
        .write()
        .await
        .get_mut::<Game>()
        .map(|g| g.end_turn(&performer, &guesser))
        .ok_or(Error::NoGame)?
        .map_err(Error::GameError)?;
    Ok(())
}


#[group]
#[commands(reset, add_players, status, list_players, add_clue, join,
           debug_mode, start_game, next_turn, start_turn)]
struct Yeats;

struct Handler;

impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("yeats", log::LevelFilter::Info)
        .init()
        .expect("Couldn't init logger");

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Coulnd't get discord token");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")
                   .no_dm_prefix(true))
        .group(&YEATS_GROUP);

    let mut client = ClientBuilder::new(token)
        .type_map(TypeMap::new())
        .type_map_insert::<Game>(Game::new())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("client go poo poo");

    log::info!("Starting client...");
    client.start()
        .await
        .expect("client start poo poo bum");
}
