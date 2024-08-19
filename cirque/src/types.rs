use std::collections::HashSet;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::server_to_client;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserID(uuid::Uuid);

impl UserID {
    pub(crate) fn generate() -> Self {
        UserID(uuid::Uuid::new_v4())
    }
}

pub type ChannelID = String;

#[derive(Debug)]
pub struct RegisteredUser {
    pub(crate) user_id: UserID,
    pub(crate) nickname: String,
    pub(crate) username: String,
    mailbox: tokio::sync::mpsc::UnboundedSender<server_to_client::Message>,
}

impl RegisteredUser {
    pub(crate) fn send(&self, message: &server_to_client::Message) {
        let _ = self.mailbox.send(message.clone());
    }

    pub(crate) fn fullspec(&self) -> String {
        format!("{}!{}@hidden", self.nickname, self.username)
    }
}

#[derive(Debug)]
pub(crate) struct RegisteringUser {
    pub(crate) user_id: UserID,
    pub(crate) nickname: Option<String>,
    pub(crate) username: Option<String>,
    mailbox: UnboundedSender<server_to_client::Message>,
}

impl RegisteringUser {
    pub(crate) fn new() -> (Self, UnboundedReceiver<server_to_client::Message>) {
        let user_id = UserID::generate();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let user = Self {
            user_id,
            nickname: None,
            username: None,
            mailbox: tx,
        };
        (user, rx)
    }

    pub(crate) fn is_ready(&self) -> bool {
        self.nickname.is_some() && self.username.is_some()
    }
}

impl From<RegisteringUser> for RegisteredUser {
    fn from(value: RegisteringUser) -> Self {
        assert!(value.is_ready());
        Self {
            user_id: value.user_id,
            nickname: value.nickname.unwrap(),
            username: value.username.unwrap(),
            mailbox: value.mailbox,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Topic {
    pub content: Vec<u8>,
    pub ts: u64,
    pub from_nickname: String,
}

impl Topic {
    pub(crate) fn is_valid(&self) -> bool {
        !self.content.is_empty() && self.ts > 0
    }
}

#[derive(Debug, Default)]
pub struct Channel {
    pub(crate) topic: Topic,
    pub(crate) users: HashSet<UserID>,
}
