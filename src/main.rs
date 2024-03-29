use std::collections::HashSet;
use tokio::time::{Duration, sleep};
use async_trait::async_trait;
use serenity::{
    prelude::*,
    model::prelude::*,
    client::ClientBuilder,
    framework::standard::{
        Args,
        StandardFramework,
        CommandResult,
        CommandGroup,
        HelpOptions,
        macros::{command, group, help},
        help_commands::plain,
    },
};
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

/// Print the status of the game
#[command]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    Executor::new(ctx, msg)
        .read(|g| ResponseOk::new(ctx, msg)
              .with_content(g.status()))
        .await
        .send()
        .await
}

/// Reset the game back to nothing - WARNING clears all clues and players
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

/// Join the game
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

/// Add a clue to the bowl. This can only be done before the game states. You should 
/// direct message your clues to Yeats, if you !add-clue in channel everyone will see
/// (Yeats will try to delete your message as quickly as possible and remind you to
/// DM them, but others may still see your secrets)
#[command]
#[aliases("add-clue")]
async fn add_clue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = args.rest().into();
    if msg.is_private() {
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
        msg.delete(ctx)
            .await?;
        let channel = msg.channel(ctx).await
            .ok_or(Error::NoChannel)?
            .guild()
            .ok_or(Error::NotAGuildChannel)?;
        ResponseOk::new(ctx, &msg)
            .with_content(format!("{} Umm... you're supposed to dm that to me", msg.author))
            .with_channel(channel)
            .send()
            .await
            .or_else(|e| {
                log::warn!("{}", e);
                Err(e)
            })
    }
}

/// Starts the game. After the game as started no more players can join
/// nor can clues be added to the bowl.
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

/// Gets the next turn ready and tags the players involved so they no to 
/// put on their drama hat
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

/// Starts the timer and draws the first clue for the performer. Clues are DM'd
/// to the performer, and when they feel the guesser has got the answer close enough
/// they should tell the guesser they're correct, and DM yeats "y", "Y" or "next-clue"
/// or the message the channel "!y", "!Y" or "!next-clue" to draw the next clue
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
                // Not sure we should ever get to this state, maybe
                // we should deliberately error if we do
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
    sleep(Duration::from_secs(50)).await;
    Executor::new(ctx, msg)
        .try_read(|g| {
            let channel = g.main_channel.clone().ok_or(Error::NoChannel)?;
            Ok(ResponseOk::new(ctx, msg)
               .with_content("TEN SECONDS LEFT!!".to_string())
               .with_channel(channel))
        })
        .await
        .send()
        .await?;
    sleep(Duration::from_secs(10)).await;
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

/// Draws the next clue if you're the current performer, otherwise tells you to piss off.
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

/// Once the bowl has run out of clues, it's time for the next round. All the clues
/// are put back into the bowl and the turn order (as well as the performer/guesser pairs)
/// are shuffled. If it's not time to start a new round you'll be told so.
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

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = plain(context, msg, args, &help_options, groups, owners).await;
    Ok(())
}

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

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("yeats", log::LevelFilter::Debug)
        .init()
        .expect("Couldn't init logger");

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Couldn't get discord token");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")
                   .no_dm_prefix(true))
        .help(&MY_HELP)
        .group(&YEATS_GROUP);

    let mut client = ClientBuilder::new(token)
        .type_map(TypeMap::new())
        .type_map_insert::<Game>(Game::new())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Couldn't build client");

    log::info!("Starting client...");
    client.start()
        .await
        .expect("Couldn't start client");
}
