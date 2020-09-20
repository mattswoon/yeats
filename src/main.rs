use serenity::{
    prelude::*,
    model::prelude::*,
    client::ClientBuilder,
};
use log::warn;
use itertools::Itertools;
use yeats::game::{
    game::Game,
    player::Player
};


#[allow(dead_code)]
enum CommandResponse {
    SuccessReacc,
    SuccessMsg(String),
    FailReacc,
    FailMsg(String),
    Nothing
}


struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut data = ctx.data.write().await;
        let game = data.get_mut::<Game>()
            .expect("There's supposed to be a game here");
        let mut tail = msg.content.split(" ");
        let head = tail.next();
        let response = match head {
            Some("!reset") => reset_game(game).await,
            Some("!add-players") => add_players(game, &msg.mentions).await,
            Some("!list-players") => list_players(game).await,
            _ => CommandResponse::Nothing 
        };

        let res_err = match response {
            CommandResponse::SuccessReacc => msg.react(&ctx, '👍').await.map(|_| ()),
            CommandResponse::SuccessMsg(s) => msg.reply(&ctx, s).await.map(|_| ()),
            CommandResponse::FailReacc => msg.react(&ctx, '🙅').await.map(|_| ()),
            CommandResponse::FailMsg(s) => msg.reply(&ctx, s).await.map(|_| ()),
            CommandResponse::Nothing => Ok(())
        };
        if let Err(why) = res_err {
            warn!("Couldn't respond because {}", why);
        };
    }
}

async fn reset_game(game: &mut Game) -> CommandResponse {
    *game = Game::new();
    CommandResponse::SuccessReacc
}

async fn add_players(game: &mut Game, users: &Vec<User>) -> CommandResponse {
    let mut players: Vec<_> = users.iter()
        .map(|u| Player::from_user(u))
        .collect();
    match game.add_players(&mut players) {
        Ok(_) => CommandResponse::SuccessReacc,
        Err(_) => CommandResponse::FailMsg("Can't add any more players, the game has already started!".to_string())
    }
}

async fn list_players(game: &mut Game) -> CommandResponse {
    CommandResponse::SuccessMsg(
        game.players
        .iter()
        .map(|p| p.name.clone())
        .join(", ")
    )
}


async fn main() {
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Coulnd't get discord token");

    let mut client = ClientBuilder::new(token)
        .type_map(TypeMap::new())
        .type_map_insert::<Game>(Game::new())
        .event_handler(Handler)
        .await
        .expect("client go poo poo");

    client.start()
        .await
        .expect("client start poo poo bum");
}
