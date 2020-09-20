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
    clue::Clue
};

#[allow(dead_code)]
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
async fn reset(ctx: &Context, msg: &Message) -> CommandResult {
    let _game = ctx.data.write()
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
    let _game = ctx.data.write()
        .await
        .get_mut::<Game>()
        .map(|g| g.add_players(&mut players))
        .ok_or(Error::NoGame)?;
    msg.react(ctx, '👍').await?;
    Ok(())
}

#[command]
#[aliases("list-players")]
async fn list_players(ctx: &Context, msg: &Message) -> CommandResult {
    let reply = ctx.data.read()
        .await
        .get::<Game>()
        .map(|g| g.players.iter().map(|p| p.name.clone()).join(", "))
        .ok_or(Error::NoGame)?;
    msg.react(ctx, '👍').await?;
    msg.reply(ctx, format!("Players in game: {}", reply)).await?;
    Ok(())
}

#[command]
#[aliases("add-clue")]
async fn add_clue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.is_private() {
        let clue_text = args.rest();
        let player: Player = (&msg.author).into();
        let _game = ctx.data.write()
            .await
            .get_mut::<Game>()
            .map(|g| {
                g.add_clue(Clue { entered_by: player,
                                  text: clue_text.into() })
            });
        Ok(())
    } else {
        msg.reply(ctx, format!("Umm... you're supposed to dm that to me")).await?;
        Ok(())
    }
}


#[group]
#[commands(reset, add_players)]
struct Yeats;

struct Handler;

impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Coulnd't get discord token");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&YEATS_GROUP);

    let mut client = ClientBuilder::new(token)
        .type_map(TypeMap::new())
        .type_map_insert::<Game>(Game::new())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("client go poo poo");

    client.start()
        .await
        .expect("client start poo poo bum");
}
