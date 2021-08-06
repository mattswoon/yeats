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
    respond2::{
        Respondable,
        OrSend,
        OrLog,
        Executor,
        ResponseOk,
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
    Executor::new(ctx, msg)
        .read(|g| ResponseOk::new(ctx, msg)
              .with_content(g.status()))
        .await
        .send()
        .await
}

#[command]
async fn reset(ctx: &Context, msg: &Message) -> CommandResult {
    Executor::new(ctx, msg)
        .write(|g| {
            log::info!("{} reset the game", msg.author.name);
            g.reset();
            ResponseOk::new(ctx, msg)
                .with_react('👍')})
        .await
        .send()
        .await
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    Executor::new(ctx, msg)
        .try_write(|g| {
            g.add_player((&msg.author).into())?;
            log::info!("{} joined the game", msg.author);
            Ok(ResponseOk::new(ctx, msg)
                .with_content(format!("{} joined the game", msg.author.name))) })
        .await
        .send()
        .await
}

#[command]
#[aliases("add-clue")]
async fn add_clue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.is_private() {
        let text = args.rest().into();
        let entered_by = (&msg.author).into();
        Executor::new(ctx, msg)
            .try_write(|g| {
                let clue = Clue { entered_by, text };
                g.add_clue(&clue)?;
                Ok(ResponseOk::new(ctx, msg)
                   .with_content(format!("The clue has been added to the bowl:\n```\n{}\n```", clue.text))) })
            .await
            .send()
            .await
    } else {
        ResponseOk::new(ctx, msg)
            .with_content("Umm... you're supposed to dm that to me".to_string())
            .send()
            .await
    }
}

#[command]
#[aliases("start-game")]
async fn start_game(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Starting game");

    let channel = msg.channel(ctx)
        .await
        .ok_or(Error::NoChannel)
        .and_then(|c| c.guild().ok_or(Error::NotAGuildChannel));

    Executor::new(ctx, msg)
        .try_write(|g| {
            let channel = channel?;
            g.start_game(channel.clone())?;
            Ok(ResponseOk::new(ctx, msg)
               .with_channel(channel)
               .with_content("Starting game".to_string()))
        })
        .await
        .send()
        .await
}

#[command]
#[aliases("next-turn")]
async fn next_turn(ctx: &Context, msg: &Message) -> CommandResult {
    let (turn, channel) = Executor::new(ctx, msg)
        .try_write_and_get(|g| {
            let channel = g.main_channel
                .clone()
                .ok_or(Error::NoChannel)?;
            let turn = g.prepare_turn()?;
            Ok((turn, channel))
        })
        .await
        .or_send()
        .await?;
    log::debug!("{:?}", &turn);
    ResponseOk::new(ctx, msg)
        .with_channel(channel)
        .with_content(format!(
            "Get ready! {} will be performing for {}",
            turn.performer,
            turn.guesser
            ))
        .send()
        .await
}

#[command]
#[aliases("start-turn")]
async fn start_turn(ctx: &Context, msg: &Message) -> CommandResult {
    let (Turn { performer, guesser, .. }, round_number) = Executor::new(ctx, msg)
        .try_write_and_get(|g| {
            g.start_turn()
        })
    .await
    .or_send()
    .await?;
    // Send a clue
    
    let dm_chan = performer.user
        .create_dm_channel(ctx)
        .await
        .or_else(|e| {
            log::warn!("{}", &e);
            Err(e)
        })?;
    
    Executor::new(ctx, msg)
        .try_write(|g| {
            let DrawClue { clue, .. } = g.draw_clue(&performer)?;
            if let Some(clue) = clue {
                Ok(ResponseOk::new(ctx, msg)
                    .with_dm_channel(dm_chan)
                    .with_content(format!("Your clue is:\n{}", clue)))
            } else {
                g.end_turn(&performer, &guesser, round_number)?;
                let reply = g.turn_summary()?.to_string();
                let channel = g.main_channel.clone().ok_or(Error::NoChannel)?;
                Ok(ResponseOk::new(ctx, msg)
                    .with_channel(channel)
                    .with_content(format!("Turn's over because the bowl is empty. Well done {} and {}, you solved the following clues:\n{}",
                                          &performer,
                                          &guesser,
                                          reply))
                    .with_redact_after(20))
            }
        })
        .await
        .send()
        .await?;

    log::info!("Starting timer for {} -> {}", &performer, &guesser);
    sleep(Duration::from_secs(60)).await;
    log::info!("Times up for {} -> {}", &performer, &guesser);

    let reply: String = Executor::new(ctx, msg)
        .try_write_and_get(|g| {
            g.end_turn(&performer, &guesser, round_number)
                .and_then(|()| g.turn_summary().map(|s| s.to_string()))
        })
        .await
        .or_log()?;

    Executor::new(ctx, msg)
        .try_read(|g| {
            let channel = g.main_channel.clone().ok_or(Error::NoChannel)?;
            Ok(ResponseOk::new(ctx, msg)
                .with_channel(channel)
                .with_content(format!("Time's up! {} and {}, you solved the following clues:\n{}",
                                      &performer,
                                      &guesser,
                                      reply))
                .with_redact_after(20))
        })
        .await
        .send()
        .await
}

#[command]
#[aliases("next-clue", "y", "Y")]
async fn next_clue(ctx: &Context, msg: &Message) -> CommandResult {
    let by: Player = (&msg.author).into();
    let dm_chan = by.user
        .create_dm_channel(ctx)
        .await
        .or_else(|e| {
            log::warn!("{}", &e);
            Err(e)
        })?;
    
    Executor::new(ctx, msg)
        .try_write(|g| {
            let DrawClue { clue, performer, guesser } = g.draw_clue(&by)?;
            if let Some(clue) = clue {
                Ok(ResponseOk::new(ctx, msg)
                    .with_dm_channel(dm_chan)
                    .with_content(format!("Your clue is:\n{}", clue)))
            } else {
                let round_number = g.current_round_number()
                    .ok_or(Error::NoRound)?;
                g.end_turn(&performer, &guesser, round_number)?;
                let reply = g.turn_summary()?.to_string();
                let channel = g.main_channel.clone().ok_or(Error::NoChannel)?;
                Ok(ResponseOk::new(ctx, msg)
                    .with_channel(channel)
                    .with_content(format!("Turn's over because the bowl is empty. Well done {} and {}, you solved the following clues:\n{}",
                                          &performer,
                                          &guesser,
                                          reply))
                    .with_redact_after(20))
            }
        })
        .await
        .send()
        .await
}

#[command]
#[aliases("next-round")]
async fn next_round(ctx: &Context, msg: &Message) -> CommandResult {
    Executor::new(ctx, msg)
        .try_write(|g| {
            g.advance_game()?;
            Ok(ResponseOk::new(ctx, msg)
               .with_content(g.status()))
        })
        .await
        .send()
        .await
}

//async fn end_turn(ctx: &Context, performer: &Player, guesser: &Player) -> Result<(), Error> {
//    ctx.data
//        .write()
//        .await
//        .get_mut::<Game>()
//        .ok_or(Error::NoGame)
//        .and_then(|g| {
//            g.end_turn(performer, guesser)
//                .map_err(Error::GameError)})
//}
//
#[group]
#[commands(
    status, 
    reset, 
    join, 
    add_clue, 
    start_game, 
    next_turn,
    start_turn,
    next_clue,
    next_round,
)]
struct Yeats;

struct Handler;

impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("yeats", log::LevelFilter::Debug)
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
