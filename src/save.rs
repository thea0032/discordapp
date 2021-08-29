use serenity::model::{channel::{Channel, GuildChannel}, id::{GuildId, MessageId}};

use crate::{message::{LoadedMessage, UserDict}, servers::Unread};

    pub struct ServerSave {
        pub labels: Vec<String>,
        pub unread: Vec<Unread>,
        pub contents: Vec<CategorySave>,
        pub dict: UserDict,
    }
    pub struct CategorySave {
        pub labels: Vec<String>,
        pub unread: Vec<Unread>,
        pub contents: Vec<ChannelSave>,
        pub s_id: Option<GuildId>,
    }
    pub struct ChannelSave {
        pub labels: Vec<String>,
        pub unread: Vec<Unread>,
        pub contents: Vec<MessageSave>,
        pub id: Option<GuildChannel>, // this is a CATEGORY, not a channel
    }
pub enum MessageSave {
    Unloaded(Channel),
    Loaded(LoadedMessageSave),
    Nonexistent,
}

pub struct LoadedMessageSave {
    pub labels: Vec<LoadedMessage>, // main vec contains messages, other vec contains lines
    pub unread: usize, // the first unread message
    pub id: Channel,
    pub more_before: Option<MessageId>, // more_after will be assumed to be true
}