use std::{collections::VecDeque, fs, io::stdout, sync::mpsc::{Receiver, Sender, channel}, time::{Duration, Instant}};

use crate::{messages::LoadingState, task::Control};
use serenity::{Client, model::{
    channel::{Channel, GuildChannel},
    id::{GuildId},
}};
use serde_json::to_string;
use serde_json::from_str;
use crate::{categories::{Categories, CategoryLabel}, channels::{ChannelLabel, Channels}, file::FileOptions, grid::Grid, input::{Parser, Response, State}, message::{LoadedMessage, UserDict}, messages::{LoadedMessages, Messages}, servers::{ServerLabel, Servers, Unread}, task::{Product, Task}, textbox::Textbox};

pub const PATH:&str = "save.ignore";

pub struct Return(pub String, pub Receiver<Response>, pub Client, pub Sender<Task>, pub Sender<Control>, pub Receiver<Product>);
pub fn load(input_server: Receiver<Response>, client: Client, tasks: Sender<Task>, control: Sender<Control>, products: Receiver<Product>) -> Result<Parser, Return> {
    let bytes = match fs::read_to_string(PATH) {
        Ok(val) => val,
        Err(why) => return Err(Return(why.to_string(), input_server, client, tasks, control, products)),
    };
    match from_str::<ParserSave>(&bytes) {
        Ok(val) => Ok(Parser::from_save(val, input_server, client, tasks, control, products)),
        Err(why) => Err(Return(why.to_string(), input_server, client, tasks, control, products)),
    }
}
pub fn save(parse: &Parser) -> Result<(), String>{
    let v = ParserSave::process(parse);
    let bytes = to_string(&v).map_err(|x| x.to_string())?;
    fs::write(PATH, bytes).map_err(|x| x.to_string())?;
    Ok(())
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ParserSave {
    pub user_dict: UserDict,
    pub file_options: FileOptions,
    pub servers: ServerSave,
}
impl ParserSave {
    pub fn process(orig: &Parser) -> ParserSave {
        ParserSave {
            user_dict: orig.int.user_dict.clone(),
            file_options: orig.int.file_options.clone(),
            servers: ServerSave::process(&orig.servers),
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ServerSave {
    pub labels: Vec<ServerLabel>,
    pub unread: Vec<Unread>,
    pub contents: Vec<CategorySave>,
}
impl ServerSave {
    pub fn process(servers: &Servers) -> ServerSave {
        ServerSave {
            labels: servers.labels.clone(),
            unread: servers.unread.clone(),
            contents: servers.contents.iter().map(|x| CategorySave::process(x)).collect()
        }
    }
    pub fn reload(self) -> Servers {
        Servers {
            labels: self.labels,
            unread: self.unread,
            contents: self.contents.into_iter().map(|x| x.reload()).collect(),
            current: 0,
            selected: 0,
            flag: true,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CategorySave {
    pub labels: Vec<CategoryLabel>,
    pub unread: Vec<Unread>,
    pub contents: Vec<ChannelSave>,
    pub s_id: Option<GuildId>,
    pub current: usize,
    pub selected: usize,
}
impl CategorySave {
    pub fn process(categories: &Categories) -> CategorySave {
        CategorySave {
            labels: categories.labels.clone(),
            unread: categories.unread.clone(),
            s_id: categories.s_id,
            current: categories.current,
            selected: categories.selected,
            contents: categories.contents.iter().map(|x| ChannelSave::process(x)).collect(),
        }
    }
    pub fn reload(self) -> Categories {
        Categories {
            labels: self.labels,
            unread: self.unread,
            contents: self.contents.into_iter().map(|x| x.reload()).collect(),
            current: self.current,
            selected: self.selected,
            s_id: self.s_id,
            flag: true,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChannelSave {
    pub labels: Vec<ChannelLabel>,
    pub unread: Vec<Unread>,
    pub contents: Vec<MessagesSave>,
    pub current: usize,
    pub selected: usize,
    pub id: Option<GuildChannel>, // this is a CATEGORY, not a channel
}
impl ChannelSave {
    pub fn process(channels: &Channels) -> ChannelSave {
        ChannelSave {
            labels: channels.labels.clone(),
            unread: channels.unread.clone(),
            id: channels.id.clone(),
            current: channels.current,
            selected: channels.selected,
            contents: channels.contents.iter().map(|x| MessagesSave::process(x)).collect()
        }
    }
    pub fn reload(self) -> Channels {
        Channels {
            labels: self.labels,
            unread: self.unread,
            contents: self.contents.into_iter().map(|x| x.reload()).collect(),
            current: self.current,
            selected: self.selected,
            id: self.id,
            flag: true,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum MessagesSave {
    Unloaded(Channel),
    Loaded(LoadedMessagesSave),
    Nonexistent,
}
impl MessagesSave {
    pub fn process(messages: &Messages) -> MessagesSave {
        match messages {
            Messages::Unloaded(val) => MessagesSave::Unloaded((*val).clone()),
            Messages::Nonexistent => MessagesSave::Nonexistent,
            Messages::Loaded(val) => MessagesSave::Loaded(LoadedMessagesSave::process(val)),
            Messages::Loading(_) => panic!("Illegal savestate!"),
        }
    }
    pub fn reload(self) -> Messages {
        match self {
            MessagesSave::Unloaded(ch) => Messages::Unloaded(ch),
            MessagesSave::Loaded(val) => Messages::Loaded(val.reload()),
            MessagesSave::Nonexistent => Messages::Nonexistent,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LoadedMessagesSave {
    pub labels: VecDeque<LoadedMessage>, // main vec contains messages, other vec contains lines
    pub unread: usize,              // the first unread message
    pub id: Channel,
    pub more_before: bool, // more_after will be assumed to be true
    pub current: usize,
    pub current_in_message: usize,
    pub selected: usize,
}
impl LoadedMessagesSave {
    pub fn process(loaded_messsages: &LoadedMessages) -> LoadedMessagesSave {
        LoadedMessagesSave {
            labels: loaded_messsages.labels.clone(),
            unread: loaded_messsages.unread,
            id: loaded_messsages.id.clone(),
            more_before: matches!(loaded_messsages.before, LoadingState::Unloaded),
            current: loaded_messsages.current,
            current_in_message: loaded_messsages.current_in_message,
            selected: loaded_messsages.selected,
        }
    }
    pub fn reload(self) -> LoadedMessages {
        LoadedMessages {
            labels: self.labels,
            unread: self.unread,
            id: self.id,
            before: if self.more_before {LoadingState::Unloaded} else {LoadingState::Finished},
            current: self.current,
            current_in_message: self.current_in_message,
            selected: self.selected,
            after: LoadingState::Unloaded,
            flag: true,
        }
    }
}
pub struct Autosave {
    last_time: Instant
}
impl Autosave {
    pub const FREQUENCY:Duration = Duration::from_secs(120);
    pub fn should_save(&self) -> bool {
        Instant::now() - self.last_time > Autosave::FREQUENCY
    }
    pub fn save(parser: &Parser) -> Autosave {
        save(parser).expect("Failed to save!");
        Autosave::new()
    }
    pub fn new() -> Autosave {
        Autosave {last_time: Instant::now()}
    }
}