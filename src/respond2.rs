use tokio::time::{Duration, sleep};
use serenity::{
    prelude::TypeMapKey,
    client::Context,
    framework::standard::CommandResult,
    model::{
        user::User,
        channel::{
            GuildChannel,
            Message,
            PrivateChannel,
        }
    }
};
use async_trait::async_trait;
use crate::{
    error::Error,
    game::game::Game,
};

pub type RespondableResult<'a> = Result<ResponseOk<'a>, ResponseErr<'a>>;
pub type DynRespondable = Box<dyn Respondable>;

#[async_trait]
pub trait Respondable {
    async fn send(self) -> CommandResult;
}

#[derive(Clone)]
pub struct Executor<'a> {
    context: &'a Context,
    message: &'a Message,
}

impl<'a> Executor<'a> {
    pub fn new(context: &'a Context, message: &'a Message) -> Executor<'a> {
        Executor { context, message }
    }

    pub async fn write<F, R>(self, action: F) -> Result<R, ResponseErr<'a>> 
    where
        F: Send + FnOnce(&mut Game) -> R,
        R: 'a + Respondable
    {
        self.context
            .data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(ResponseErr::new(self.context, self.message, Error::NoGame))
            .map(action)
    }
    
    pub async fn write_and_get<F, T>(self, action: F) -> Result<T, ResponseErr<'a>> 
    where
        F: Send + FnOnce(&mut Game) -> T,
    {
        self.context
            .data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(ResponseErr::new(self.context, self.message, Error::NoGame))
            .map(action)
    }

    pub async fn try_write<F, R>(self, action: F) -> Result<R, ResponseErr<'a>>
    where
        F: Send + FnOnce(&mut Game) -> Result<R, Error>,
        R: 'a + Respondable
    {
        self.context
            .data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(action)
            .map_err(|e| ResponseErr::new(self.context, self.message, e))
    }
    
    pub async fn try_write_and_get<F, T>(self, action: F) -> Result<T, ResponseErr<'a>>
    where
        F: Send + FnOnce(&mut Game) -> Result<T, Error>,
    {
        self.context
            .data
            .write()
            .await
            .get_mut::<Game>()
            .ok_or(Error::NoGame)
            .and_then(action)
            .map_err(|e| ResponseErr::new(self.context, self.message, e))
    }
    
    pub async fn read<F, R>(&self, action: F) -> Result<R, ResponseErr<'a>> 
    where
        F: Send + Fn(&Game) -> R,
        R: 'a + Respondable,
    {
        self.context
            .data
            .read()
            .await
            .get::<Game>()
            .ok_or(ResponseErr::new(self.context, self.message, Error::NoGame))
            .map(action)
    }
    
    pub async fn try_read<F, R>(self, action: F) -> Result<R, ResponseErr<'a>>
    where
        F: Send + Fn(&Game) -> Result<R, Error>,
        R: 'a + Respondable
    {
        self.context
            .data
            .read()
            .await
            .get::<Game>()
            .ok_or(Error::NoGame)
            .and_then(action)
            .map_err(|e| ResponseErr::new(self.context, self.message, e))
    }

    pub async fn get<T, F>(&self, action: F) -> Result<T, ResponseErr<'a>> 
    where
        F: Send + Fn(&Game) -> T,
    {
        self.context
            .data
            .read()
            .await
            .get::<Game>()
            .ok_or(ResponseErr::new(self.context, self.message, Error::NoGame))
            .map(action)
    }
    
    pub async fn try_get<T, F>(&self, action: F) -> Result<T, ResponseErr<'a>> 
    where
        F: Send + Fn(&Game) -> Result<T, Error>,
    {
        self.context
            .data
            .read()
            .await
            .get::<Game>()
            .ok_or(Error::NoGame)
            .and_then(action)
            .map_err(|e| ResponseErr::new(self.context, self.message, e))
    }
}

/// If something returns `Result<(), Error>` that means we don't respond
/// in discord, even if it errors
#[async_trait]
impl Respondable for () {
    async fn send(self) -> CommandResult {
        Ok(())
    }
}

#[derive(Clone)]
pub struct ResponseErr<'a> {
    pub context: &'a Context,
    pub message: &'a Message,
    pub error: Error
}

impl<'a> ResponseErr<'a> {
    pub fn new(context: &'a Context, message: &'a Message, error : Error) -> ResponseErr<'a> {
        ResponseErr {
            context,
            message,
            error
        }
    }
}

pub struct ResponseOk<'a> {
    pub context: &'a Context,
    pub message: &'a Message,
    pub channel: Option<GuildChannel>,
    pub dm_channel: Option<PrivateChannel>,
    pub react: Option<char>,
    pub content: Option<String>,
    pub redact_after: Option<u64>,
}

impl<'a> ResponseOk<'a> {
    pub fn new(context: &'a Context, message: &'a Message) -> ResponseOk<'a> {
        ResponseOk {
            context, 
            message, 
            channel: None,
            dm_channel: None,
            react: None,
            content: None,
            redact_after: None,
        }
    }

    pub fn with_channel(self, channel: GuildChannel) -> ResponseOk<'a> {
        ResponseOk{
            channel: Some(channel),
            ..self
        }
    }

    pub fn with_dm_channel(self, dm_channel: PrivateChannel) -> ResponseOk<'a> {
        ResponseOk {
            dm_channel: Some(dm_channel),
            ..self
        }
    }

    pub fn with_react(self, react: char) -> ResponseOk<'a> {
        ResponseOk{
            react: Some(react),
            ..self
        }
    }
    
    pub fn with_content(self, content: String) -> ResponseOk<'a> {
        ResponseOk{
            content: Some(content),
            ..self
        }
    }

    pub fn with_redact_after(self, redact_after: u64) -> ResponseOk<'a> {
        ResponseOk {
            redact_after: Some(redact_after),
            ..self
        }
    }
}

#[async_trait]
impl<'a> Respondable for ResponseOk<'a> {
    async fn send(self) -> CommandResult {
        if let Some(r) = self.react {
            self.message.react(self.context, r).await?;
        }
        if let Some(text) = self.content {
            let mut message = match (self.channel, self.dm_channel) {
                (Some(chan), None) => {
                    chan.send_message(self.context, |m| m.content(&text))
                        .await
                },
                (None, Some(dm_chan)) => {
                    dm_chan.send_message(self.context, |m| m.content(&text))
                        .await
                },
                (Some(chan), Some(dm_chan)) => {
                    chan.send_message(self.context, |m| m.content(&text))
                        .await?;
                    dm_chan.send_message(self.context, |m| m.content(&text))
                        .await
                },
                (None, None) => {
                    self.message.reply(self.context, text)
                        .await
                }
            }?;
            if let Some(redact_after) = self.redact_after {
                sleep(Duration::from_secs(redact_after)).await;
                message.edit(self.context, |m| m.content("*REDACTED*"))
                    .await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> Respondable for ResponseErr<'a> {
    async fn send(self) -> CommandResult {
        log::warn!("{}: {}", self.message.author, self.error);
        self.message.reply(self.context, self.error)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<T: Respondable + Send, E: Respondable + Send> Respondable for Result<T, E> {
    async fn send(self) -> CommandResult {
        match self {
            Ok(o) => o.send(),
            Err(e) => e.send(),
        }.await
    }
}

#[async_trait]
pub trait OrSend {
    type OkType;

    async fn or_send(self) -> CommandResult<Self::OkType>;
}

#[async_trait]
impl<T> OrSend for Result<T, ResponseErr<'_>> 
where 
    T: Send
{
    type OkType = T;

    async fn or_send(self) -> CommandResult<T> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let err = e.error.clone();
                e.send().await?;
                Err(Box::new(err))
            }
        }
    }
}

pub trait OrLog {
    type OkType;

    fn or_log(self) -> CommandResult<Self::OkType>;
}

impl<T> OrLog for Result<T, ResponseErr<'_>>
where
    T: Send
{
    type OkType = T;

    fn or_log(self) -> CommandResult<T> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                log::warn!("{}", &e.error);
                Err(Box::new(e.error))
            }
        }
    }
}
