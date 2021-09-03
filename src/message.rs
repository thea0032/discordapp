use std::collections::{HashMap, LinkedList};

use chrono::{DateTime, Local};
use futures::executor::block_on;
use serenity::model::{
    channel::{Attachment, Embed},
    id::{MessageId, UserId},
};

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
}
impl LoadedMessage {
    pub fn from_content(
        name: UserId,
        s: Vec<String>,
        time: DateTime<Local>,
        id: MessageId,
    ) -> Self {
        LoadedMessage {
            content: LoadedMessageInstance::new(s, time),
            prev: LinkedList::new(),
            next: LinkedList::new(),
            user: name,
            first_time: time,
            id,
        }
    }
    pub fn push_content(&mut self, s: Vec<String>, time: DateTime<Local>) {
        self.next.push_front(LoadedMessageInstance::new(s, time));
    } // used for messages with edit history
    pub fn last(&mut self) -> &mut LoadedMessageInstance {
        self.next.front_mut().unwrap_or(&mut self.content)
    }
    pub fn attachment(mut self, v: Attachment) -> Self {
        let url = process(v.url.clone());
        let name = v.filename.clone();
        let (location, should_download) = fs_write(&url).expect("FAILED");
        if should_download {
            let result = tokio::runtime::Runtime::new()
                .expect("Could not create a runtime!")
                .block_on(v.download())
                .unwrap_or("could not download!".to_string().into_bytes());
            fs_write_2(result, &url);
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
            return crate::ansi::COLORS[info.color].to_string()
                + &*info.name.chars().take(len).collect::<String>();
        } else {
            return crate::ansi::COLORS[info.color].to_string()
                + &*(info.name.clone() + " " + &*format_time(self.content.time))
                    .chars()
                    .take(len)
                    .collect::<String>();
        }
    }
    pub fn change_color(&self, dict: &mut UserDict) {
        let mut info = (&dict.contents[&self.user]).clone();
        info.color += 1;
        info.color %= COLORS.len();
        dict.contents.insert(self.user, info);
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserInfo {
    pub name: String,
    pub color: usize,
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
pub fn process(s: String) -> String {
    let temp = s.replace("/", "").replace("\\", "").replace(":", "");
    let temp = temp.split(".").collect::<Vec<_>>();
    let mut temp = temp.iter().rev();
    let first = temp.next().unwrap();
    let second = temp.next().unwrap();
    temp.next().unwrap().to_string() + second + "." + first
}
