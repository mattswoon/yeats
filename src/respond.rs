use serenity::{
    prelude::Context,
    model::{
        user::User,
        channel::{
            Message,
            GuildChannel,
        },
    },
    framework::standard::CommandResult,
};
use crate::error::Error;

pub enum Response<'a> {
    Reply { to: &'a Message, content: String },
    ThumbsUp { to: &'a Message },
    ProhibitedReply { to: &'a Message, content: String },
    MessageGuild { to: &'a GuildChannel, content: String },
    Dm { to: &'a User, content: String },
}

pub async fn send(ctx: &Context, response: Response<'_>) -> CommandResult {
    match response {
        Response::Reply { to, content } => { 
            to.reply(ctx, content).await?;
        },
        Response::ThumbsUp { to } => { 
            to.react(ctx, '👍').await?;
        },
        Response::ProhibitedReply { to, content } => {
            to.react(ctx, '🚫').await?;
            to.reply(ctx, content).await?;
        },
        Response::MessageGuild { to, content } => {
            to.send_message(ctx, |m| m.content(content)).await?;
        },
        Response::Dm { to, content } => {
            to.dm(ctx, |m| m.content(content)).await?;
        }
    };
    Ok(())
}

pub async fn or_reply_to_message<T>(ctx: &Context, msg: &Message, res: Result<T, Error>) -> Result<T, Error> 
{
    match res {
        Ok(_) => (),
        Err(ref e) => { send(ctx, e.reply_to_message(msg)).await.map_err(Error::Command)?; }
    };
    res
}

pub async fn or_reply_in_channel<T>(ctx: &Context, channel: &GuildChannel, res: Result<T, Error>) -> Result<T, Error> 
{
    match res {
        Ok(_) => (),
        Err(ref e) => { send(ctx, e.reply_in_channel(channel)).await.map_err(Error::Command)?; }
    };
    res
}

pub async fn or_dm_user<T>(ctx: &Context, user: &User, res: Result<T, Error>) -> Result<T, Error> 
{
    match res {
        Ok(_) => (),
        Err(ref e) => { send(ctx, e.dm_user(user)).await.map_err(Error::Command)?; }
    };
    res
}

pub trait Respondable<'a> {
    fn reply_to_message(&self, msg: &'a Message) -> Response<'a>;

    fn reply_in_channel(&self, channel: &'a GuildChannel) -> Response<'a>;

    fn dm_user(&self, user: &'a User) -> Response<'a>;
}

impl<'a> Respondable<'a> for String {
    fn reply_to_message(&self, msg: &'a Message) -> Response<'a> {
        Response::Reply { to: msg, content: self.clone() }
    }

    fn reply_in_channel(&self, channel: &'a GuildChannel) -> Response<'a> {
        Response::MessageGuild { to: channel, content: self.clone() }
    }

    fn dm_user(&self, user: &'a User) -> Response<'a> {
        Response::Dm { to: user, content: self.clone() }
    }
}

impl<'a, 'b> Respondable<'a> for &'b str {
    fn reply_to_message(&self, msg: &'a Message) -> Response<'a> {
        Response::Reply { to: msg, content: self.to_string() }
    }

    fn reply_in_channel(&self, chan: &'a GuildChannel) -> Response<'a> {
        Response::MessageGuild { to: chan, content: self.to_string() }
    }

    fn dm_user(&self, user: &'a User) -> Response<'a> {
        Response::Dm { to: user, content: self.to_string() }
    }
}

impl<'a, T: Respondable<'a>> Respondable<'a> for Option<T> {
    fn reply_to_message(&self, msg: &'a Message) -> Response<'a> {
        self.as_ref().map(|t| t.reply_to_message(msg))
            .unwrap_or(Response::ThumbsUp { to: msg })
    }

    fn reply_in_channel(&self, channel: &'a GuildChannel) -> Response<'a> {
        self.as_ref().map(|t| t.reply_in_channel(channel))
            .unwrap_or(Response::MessageGuild { to: channel, content: "👍".to_string() })
    }

    fn dm_user(&self, user: &'a User) -> Response<'a> {
        self.as_ref().map(|t| t.dm_user(user))
            .unwrap_or(Response::Dm {to: user, content: "👍".to_string() })
    }
}

impl<'a> Respondable<'a> for Error {
    fn reply_to_message(&self, msg: &'a Message) -> Response<'a> {
        log::warn!("{}", &self);
        Response::ProhibitedReply { to: msg, content: self.to_string() }
    }

    fn reply_in_channel(&self, channel: &'a GuildChannel) -> Response<'a> {
        log::warn!("{}", &self);
        Response::MessageGuild { to: channel, content: format!("🚫{}🚫", self) }
    }

    fn dm_user(&self, user: &'a User) -> Response<'a> {
        log::warn!("{}", &self);
        Response::Dm { to: user, content: format!("🚫{}🚫", &self) }
    }
}

impl<'a, 'b> Respondable<'a> for &'b Error {
    fn reply_to_message(&self, msg: &'a Message) -> Response<'a> {
        log::warn!("{}", &self);
        Response::ProhibitedReply { to: msg, content: self.to_string() }
    }

    fn reply_in_channel(&self, channel: &'a GuildChannel) -> Response<'a> {
        log::warn!("{}", &self);
        Response::MessageGuild { to: channel, content: format!("🚫{}🚫", self) }
    }

    fn dm_user(&self, user: &'a User) -> Response<'a> {
        log::warn!("{}", &self);
        Response::Dm { to: user, content: format!("🚫{}🚫", &self) }
    }
}

impl<'a, T: Respondable<'a>, E: Respondable<'a>> Respondable<'a> for Result<T, E> {
    fn reply_to_message(&self, msg: &'a Message) -> Response<'a> {
        self.as_ref().map(|t| t.reply_to_message(msg))
            .unwrap_or_else(|e| e.reply_to_message(msg))
    }

    fn reply_in_channel(&self, channel: &'a GuildChannel) -> Response<'a> {
        self.as_ref().map(|t| t.reply_in_channel(channel))
            .unwrap_or_else(|e| e.reply_in_channel(channel))
    }

    fn dm_user(&self, user: &'a User) -> Response<'a> {
        self.as_ref().map(|t| t.dm_user(user))
            .unwrap_or_else(|e| e.dm_user(user))
    }
}
