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
//use log::warn;
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
    GameError(GameError)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoGame => 
                write!(f, "No Game object in context"),
            Error::GameError(ge) => write!(f, "{}", ge),
        }
    }
}

impl std::error::Error for Error {}

#[command]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Checking status");
    msg.reply(ctx, "Status: Ok").await?;
    Ok(())
}

#[command]
async fn reset(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Resetting game");
    ctx.data.write()
        .await
        .get_mut::<Game>()
        .map(|g| *g = Game::new())
        .ok_or(Error::NoGame)?;
    msg.react(ctx, '👍').await?;
    Ok(())
}

#[command]
#[aliases("add-players")]
async fn add_players(ctx: &Context, msg: &Message) -> CommandResult {
    let mut players: Vec<Player> = msg.mentions
        .iter()
        .map(|u| u.into())
        .collect();
    log::info!("Adding players {}", &players.iter().join(", "));
    let reply = format!("Added players {}", players.iter()
                        .cloned()
                        .map(|p| p.name)
                        .collect::<Vec<_>>()
                        .join(", "));
    ctx.data.write()
        .await
        .get_mut::<Game>()
        .map(|g| g.add_players(&mut players))
        .ok_or(Error::NoGame)?
        .map_err(Error::GameError)?;
    msg.reply(ctx, reply).await?;
    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let player: Player = (&msg.author).into();
    log::info!("Adding player {} to the game", player);
    let reply = format!("Added player {} to the game", &player.name);
    ctx.data.write()
        .await
        .get_mut::<Game>()
        .map(|g| g.add_player(player))
        .ok_or(Error::NoGame)?
        .map_err(Error::GameError)?;
    msg.reply(ctx, reply).await?;
    Ok(())
}

#[command]
#[aliases("list-players")]
async fn list_players(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Listing players");
    let reply = ctx.data.read()
        .await
        .get::<Game>()
        .map(|g| g.players.iter().map(|p| p.name.clone()).join(", "))
        .ok_or(Error::NoGame)?;
    log::info!("{}", &reply);
    msg.reply(ctx, format!("Players in game: {}", reply)).await?;
    Ok(())
}

#[command]
#[aliases("add-clue")]
async fn add_clue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.is_private() {
        let clue_text = args.rest();
        let player: Player = (&msg.author).into();
        log::info!("Adding clue from {}", player);
        ctx.data.write()
            .await
            .get_mut::<Game>()
            .map(|g| {
                g.add_clue(Clue { entered_by: player,
                                  text: clue_text.into() })
            });
        msg.react(ctx, '👍').await?;
        Ok(())
    } else {
        msg.reply(ctx, "Umm... you're supposed to dm that to me").await?;
        Ok(())
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
    ctx.data
        .write()
        .await
        .get_mut::<Game>()
        .map(|g| g.start_game())
        .ok_or(Error::NoGame)?
        .map_err(Error::GameError)?;
    let reply = ctx
        .data
        .read()
        .await
        .get::<Game>()
        .map(|g| g.start_game_message())
        .ok_or(Error::NoGame)?;
    msg.reply(ctx, reply).await?;
    Ok(())
}

#[command]
#[aliases("next-turn")]
async fn next_turn(ctx: &Context, msg: &Message) -> CommandResult {
    ctx.data
        .write()
        .await
        .get_mut::<Game>()
        .map(Game::prepare_turn)
        .ok_or(Error::NoGame)?
        .map_err(Error::GameError)?;
    let reply = ctx.data
        .read()
        .await
        .get::<Game>()
        .map(Game::ready_turn_message)
        .ok_or(Error::NoGame)?
        .map_err(Error::GameError)?;
    msg.reply(ctx, reply).await?;
    Ok(())
}

#[command]
#[aliases("start-turn")]
async fn start_turn(ctx: &Context, msg: &Message) -> CommandResult {
    msg.react(ctx, '👍').await?;
    let Turn { performer, guesser, state: _ } = ctx.data
        .write()
        .await
        .get_mut::<Game>()
        .map(Game::start_turn)
        .ok_or(Error::NoGame)?
        .map_err(Error::GameError)?;
    sleep(Duration::from_secs(60)).await;
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
