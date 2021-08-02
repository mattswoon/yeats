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
use yeats::{
    error::Error,
    game::{
        game::{
            Game,
            DrawClue,
        },
        player::Player,
        clue::Clue,
        turn::Turn,
    },
    respond::{
        Respondable, 
        send,
        or_reply_to_message,
        or_reply_in_channel,
    },
};

async fn get_main_channel(ctx: &Context) -> Result<Option<GuildChannel>, Error> {
    ctx.data
        .read()
        .await
        .get::<Game>()
        .ok_or(Error::NoGame)
        .map(|g| g.main_channel.clone())
}

#[command]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is checking status", msg.author.name);
    send(ctx, ctx.data
            .read()
            .await
            .get::<Game>()
            .ok_or(Error::NoGame)
            .map(Game::status)
            .reply_to_message(msg)).await
}

#[command]
async fn reset(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is resetting the game", msg.author.name);
    send(ctx, ctx.data.write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .map(Game::reset)
            .reply_to_message(msg)).await
}

#[command]
#[aliases("add-players")]
async fn add_players(ctx: &Context, msg: &Message) -> CommandResult {
    let mut players: Vec<Player> = msg.mentions
        .iter()
        .map(|u| u.into())
        .collect();
    log::info!("{} is adding players {}", msg.author.name, &players.iter().join(", "));
    send(ctx, ctx.data.write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.add_players(&mut players).map_err(Error::GameError))
            .reply_to_message(msg)).await
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is joining the game", msg.author.name);
    let player: Player = (&msg.author).into();
    send(ctx, ctx.data
         .write()
         .await
         .get_mut::<Game>()
         .ok_or(Error::NoGame)
         .and_then(|g| g.add_player(player).map_err(Error::GameError))
         .reply_to_message(msg)).await
}

#[command]
#[aliases("list-players")]
async fn list_players(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("{} is listing players", msg.author.name);
    send(ctx, ctx.data.read()
            .await
            .get::<Game>()
            .ok_or(Error::NoGame)
            .map(Game::list_players)
            .reply_to_message(msg)).await
}

#[command]
#[aliases("add-clue")]
async fn add_clue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.is_private() {
        let text = args.rest().into();
        let entered_by: Player = (&msg.author).into();
        log::info!("Adding clue from {}", &entered_by);
        send(ctx, ctx.data
                .write()
                .await
                .get_mut::<Game>()
                .ok_or(Error::NoGame)
                .and_then(|g| g.add_clue(Clue { entered_by, text })
                          .map_err(Error::GameError))
                .reply_to_message(msg)).await
    } else {
        send(ctx, "Umm... you're supposed to dm that to me".to_string()
             .reply_to_message(msg)).await
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
            send(ctx, "Shh, it's our little secret".reply_to_message(msg)).await?;
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
    let channel = msg.channel(ctx)
        .await
        .ok_or(Error::NoChannel)
        .and_then(|c| c.guild().ok_or(Error::NotAGuildChannel));

    match channel {
        Ok(channel) => {
            send(ctx, ctx.data
                    .write()
                    .await
                    .get_mut::<Game>()
                    .ok_or(Error::NoGame)
                    .and_then(|g| g.start_game(channel).map_err(Error::GameError))
                    .reply_to_message(msg)).await
        },
        Err(e) => {
            send(ctx, (&e).reply_to_message(msg)).await?;
            Err(e.into())
        }
    }
}

#[command]
#[aliases("next-turn")]
async fn next_turn(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Queueing next turn");
    send(ctx,
        ctx.data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.prepare_turn().map_err(Error::GameError))
            .reply_to_message(msg)).await
}

#[command]
#[aliases("start-turn")]
async fn start_turn(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Starting turn");
    let channel = or_reply_to_message(
        ctx, 
        msg, 
        get_main_channel(ctx).await
            .and_then(|oc| oc.ok_or(Error::NoChannel))
    ).await?;
    // HERE
    let turn = or_reply_in_channel(
        ctx, 
        &channel,
        ctx.data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.start_turn().map_err(Error::GameError)) 
        ).await?; 

    let Turn { performer, guesser, state: _ } = turn;

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

#[command]
#[aliases("next-clue", "y", "Y")]
async fn next_clue(ctx: &Context, msg: &Message) -> CommandResult {
    let by = (&msg.author).into();
    let drawclue = or_reply_to_message(
        ctx, 
        msg, 
        ctx.data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(|g| g.draw_clue(&by).map_err(Error::GameError))).await?;

    match drawclue {
        DrawClue { clue: Some(c), .. } => {
            msg.author.dm(ctx, |m| m.content(c.text)).await?;
            Ok(())
        },
        DrawClue { clue: None, performer, guesser } => {
            end_turn(ctx, &performer, &guesser).await?;
            Ok(())
        }
    }
}

async fn end_turn(ctx: &Context, performer: &Player, guesser: &Player) -> Result<(), Error> {
    ctx.data
        .write()
        .await
        .get_mut::<Game>()
        .ok_or(Error::NoGame)
        .and_then(|g| {
            g.end_turn(performer, guesser)
                .map_err(Error::GameError)})
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
