use std::collections::{HashMap, LinkedList};

use chrono::{DateTime, Local};
use crate::{block_on::block_on, colors::Color, task::{Task, process}};
use serenity::model::{channel::{Attachment, Embed, Message}, id::{MessageId, UserId}};
use futures::channel::mpsc::Sender;

use crate::{
    ansi::COLORS,
    file::{fs_write, fs_write_2},
    format::format_time,
};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LoadedMessageInstance {
    pub content: Vec<String>,
    pub mentions: bool,
    pub attachments: Vec<String>,
    pub attachment_url: Vec<String>,
    pub time: DateTime<Local>,
}
impl LoadedMessageInstance {
    pub fn new(s: Vec<String>, time: DateTime<Local>) -> Self {
        Self {
            content: s,
            mentions: false,
            attachment_url: Vec::new(),
            attachments: Vec::new(),
            time,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LoadedMessage {
    pub content: LoadedMessageInstance,          // current
    pub prev: LinkedList<LoadedMessageInstance>, // previous edits
    pub next: LinkedList<LoadedMessageInstance>, // future edits,
    pub user: UserId,
    pub first_time: DateTime<Local>,
    pub id: MessageId,
    pub username: String,
}
impl LoadedMessage {
    pub fn from_message(msg: Message, tasks: &mut Vec<Task>) -> Self {
        let split_val = msg.content.split("\n").into_iter().map(|x| x.to_string()).collect();
        let mut v = LoadedMessage::from_content(msg.author.id, split_val, msg.timestamp.with_timezone(&Local), msg.id, msg.author.name);
        for line in msg.attachments {
            v = v.attachment(line, tasks);
        }
        v
    }
    pub fn from_content(
        name: UserId,
        s: Vec<String>,
        time: DateTime<Local>,
        id: MessageId,
        username: String,
    ) -> Self {
        LoadedMessage {
            content: LoadedMessageInstance::new(s, time),
            prev: LinkedList::new(),
            next: LinkedList::new(),
            user: name,
            first_time: time,
            id,
            username,
        }
    }
    pub fn push_content(&mut self, s: Vec<String>, time: DateTime<Local>) {
        self.next.push_front(LoadedMessageInstance::new(s, time));
    } // used for messages with edit history
    pub fn last(&mut self) -> &mut LoadedMessageInstance {
        self.next.front_mut().unwrap_or(&mut self.content)
    }
    pub fn attachment(mut self, v: Attachment, tasks: &mut Vec<Task>) -> Self {
        let url = process(v.url.clone());
        let name = v.filename.clone();
        let (location, should_download) = fs_write(&url);
        if should_download {
            tasks.push(Task::Download(v, url.clone()));
        }
        self.last().attachment_url.push(location);
        self.last().attachments.push(name);
        self
    }
    pub fn embed(mut self, v: Embed) -> Self {
        self
    }
    /*pub fn embed(mut self, v: Embed) -> Self {
        let url = v.url.clone();
        let img = v.image.unwrap().url;
        let name = v.filename.clone();
        let location = fs_write(|| block_on(v.download()).ok(), url).unwrap_or("/".to_string());
        self.last().attachment_url.push(location);
        self.last().attachments.push(name);
        self
    }*/
    pub fn user(&self, dict: &UserDict, len: usize) -> String {
        let info = &dict.contents[&self.user];
        if len < info.name.len() {
            return info.color.to_ansi_value()
                + &*info.name.chars().take(len).collect::<String>();
        } else {
            return  info.color.to_ansi_value()
                + &*(info.name.clone() + " " + &*format_time(self.content.time))
                    .chars()
                    .take(len)
                    .collect::<String>();
        }
    }
    pub fn red(&self, dict: &mut UserDict) {
        let mut info = (&dict.contents[&self.user]).clone();
        info.color.red();
        dict.contents.insert(self.user, info);
    }
    pub fn blue(&self, dict: &mut UserDict) {
        let mut info = (&dict.contents[&self.user]).clone();
        info.color.blue();
        dict.contents.insert(self.user, info);
    }
    pub fn green(&self, dict: &mut UserDict) {
        let mut info = (&dict.contents[&self.user]).clone();
        info.color.green();
        dict.contents.insert(self.user, info);
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserInfo {
    pub name: String,
    pub color: Color,
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserDict {
    pub contents: HashMap<UserId, UserInfo>,
}
impl UserDict {
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
        }
    }
}