use std::{collections::VecDeque, io::Stdout, sync::{Arc, mpsc::Sender}, time::Duration};

use chrono::Local;
use crossterm::{cursor::MoveTo, queue, style::Print};

use crate::{block_on::block_on, colors::Color, input::Response, task::{Task}};
use serenity::{Client, model::{channel::{Channel, Message, PrivateChannel}, id::MessageId}};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    ansi,
    file::ExtConfig,
    render::Grid,
    input::Parser,
    message::{LoadedMessage, UserDict, UserInfo},
};

pub enum Messages {
    Unloaded(Channel),
    Loading(Channel),
    Loaded(LoadedMessages),
    Nonexistent,
}
impl Messages {
    pub fn new() -> Self {
        Self::Nonexistent
    }
    pub fn with_channel(ch: Channel) -> Self {
        Self::Unloaded(ch)
    }
    pub fn flag(&mut self) {
        if let Messages::Loaded(val) = self {
            val.flag();
        }
    }
    pub fn id(&self) -> Option<Channel> {
        match &self {
            Messages::Unloaded(val) => Some(val.clone()),
            Messages::Loaded(val) => Some(val.id.clone()),
            Messages::Loading(val) => Some(val.clone()),
            Messages::Nonexistent => None,
        }
    }
    pub fn draw(&mut self, grid: &Grid, out: &mut Stdout, dict: &mut UserDict, tasks: &Sender<Task>) -> bool {
        match self {
            Messages::Unloaded(_) => false,
            Messages::Loading(_) => {
                queue!(out, MoveTo(grid.border_3 as u16, grid.start_y as u16));
                queue!(out, Print("This channel is currently still loading."));
                true
            },
            Messages::Loaded(val) => val.draw(grid, out, dict, tasks),
            Messages::Nonexistent => false,
        }
    }
    pub fn receive_new(&mut self, dict: &mut UserDict, tasks: &Sender<Task>, messages: Vec<LoadedMessage>, more:bool) {
        if let Messages::Loading(v) = self {
            *self = Messages::Loaded(LoadedMessages::with_finished_messages(v.clone(), messages, more, dict));
        }
    }
    pub fn update(&mut self, tasks: &Sender<Task>) {
        if let Messages::Unloaded(v) = self {
            tasks.send(Task::GetNewMessages(v.clone())).expect("Could not send!");
            *self = Messages::Loading(v.clone());
        }
    }
    /// Updates this to make sure any extra messages are included. Returns true if an update was performed, and false if no such update was.  
    pub fn update_to_end(&mut self, client: &mut Client, dict: &mut UserDict, tasks: &Sender<Task>) -> bool {
        if let Messages::Loaded(v) = self {
            if matches!(v.after, LoadingState::Unloaded) {
                // gets the message id to use as a timestamp. If there are no messages, a default of zero is used. 
                let message_id = v.labels.back().and_then(|x| Some(x.id)).unwrap_or(MessageId(0));
                let ch = v.id.clone();
                tasks.send(Task::GetMessagesAfter(ch, message_id)).expect("Could not send task!");
                v.after = LoadingState::Loading;
                true
            } else {
                false
            }
        } else {
            true
        }
    }
    pub fn assume_loaded(&mut self) -> &mut LoadedMessages {
        if let Messages::Loaded(val) = self {
            val
        } else {
            panic!("unwrap failed!")
        }
    }
    pub fn receive_message(&mut self, dict: &mut UserDict, tasks: &Sender<Task>, msg: Message) {
        match self {
            Messages::Unloaded(ch) => {
                tasks.send(Task::GetNewMessages(ch.clone()));
                tasks.send(Task::Kick(Response::Message(msg), Duration::from_millis(100)));
            },
            Messages::Loading(_) => {
                tasks.send(Task::Kick(Response::Message(msg), Duration::from_millis(100)));
            },
            Messages::Loaded(val) => {
                val.receive_message(dict, tasks, msg);
            },
            Messages::Nonexistent => todo!(),
        }
    } 
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum LoadingState {
    Finished,
    Unloaded,
    Loading
}
pub struct LoadedMessages {
    pub labels: VecDeque<LoadedMessage>, // main vec contains messages, other vec contains lines
    pub unread: usize,              // the first unread message
    pub current: usize,
    pub current_in_message: usize,
    pub selected: usize,
    pub id: Channel,
    pub before: LoadingState,
    pub after: LoadingState,
    pub flag: bool,
}
impl LoadedMessages {
    pub fn with_messages(
        id: Channel,
        messages: Vec<Message>,
        more: bool,
        dict: &mut UserDict,
        tasks: &Sender<Task>
    ) -> Self {
        let mut result = LoadedMessages::new(id);
        result.before = if more {LoadingState::Unloaded} else {LoadingState::Finished};
        for line in messages.into_iter().rev() {
            let mut task_buffer = Vec::new();
            result.add_complete(LoadedMessage::from_message(line, &mut task_buffer), None, dict);
            for line in task_buffer {
                tasks.send(line);
            }
        }
        result
    }pub fn with_finished_messages(
        id: Channel,
        messages: Vec<LoadedMessage>,
        more: bool,
        dict: &mut UserDict,
    ) -> Self {
        let mut result = LoadedMessages::new(id);
        result.before = if more {LoadingState::Unloaded} else {LoadingState::Finished};
        for line in messages.into_iter().rev() {
            result.add_complete(line, None, dict);
        }
        result
    }
    pub fn new(id: Channel) -> Self {
        LoadedMessages {
            labels: VecDeque::new(),
            unread: 0,
            current: 0,
            selected: 0,
            flag: true,
            current_in_message: 0,
            id,
            before: LoadingState::Finished,
            after: LoadingState::Finished,
        }
    }
    pub fn flag(&mut self) {
        self.flag = true;
    }
    pub fn red(&mut self, dict: &mut UserDict) {
        let v = &mut self.labels[self.current];
        self.flag = true;
        v.red(dict);
    }
    pub fn blue(&mut self, dict: &mut UserDict) {
        let v = &mut self.labels[self.current];
        self.flag = true;
        v.blue(dict);
    }
    pub fn green(&mut self, dict: &mut UserDict) {
        let v = &mut self.labels[self.current];
        self.flag = true;
        v.green(dict);
    }
    pub fn up(&mut self, grid: &Grid) {
        let cap_len = self.count(grid, self.current) - 1;
        while self.current_in_message > cap_len {
            self.current_in_message -= 1;
        }
        if self.current_in_message > 0 {
            self.current_in_message -= 1;
            self.flag = true;
        } else if self.current > 0 {
            self.current -= 1;
            self.current_in_message = self.count(grid, self.current) - 1;
            self.flag = true;
        }
    }
    pub fn down(&mut self, grid: &Grid) {
        if self.current_in_message < self.count(grid, self.current) - 1 {
            self.current_in_message += 1;
            self.flag = true;
        } else if self.current < self.labels.len() - 1 {
            self.current += 1;
            self.current_in_message = 0;
            self.flag = true;
        }
    }
    pub fn shift_up(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.flag = true;
        }
    }
    pub fn shift_down(&mut self) {
        if self.current < self.labels.len() - 1 {
            self.current += 1;
            self.flag = true;
        }
    }
    pub fn ctrl_up(&mut self) {
        self.current = 0;
        self.current_in_message = 0;
    }
    pub fn ctrl_down(&mut self, grid: &Grid) {
        self.current = self.labels.len();
        self.current_in_message = self.count(grid, self.labels.len() - 1);
    }
    pub fn select(&mut self) {
        self.selected = self.current;
        self.flag = true;
    }
    pub fn back(&mut self) {
        self.current = self.selected;
        self.flag = true;
    }
    pub fn mark(&mut self, pos: usize) {
        self.unread = pos;
        self.flag = true;
    }
    pub fn receive_message(&mut self, dict: &mut UserDict, tasks: &Sender<Task>, msg: Message) {
        match self.after {
            LoadingState::Finished => {
                let mut task_buffer = Vec::new();
                self.add_complete(crate::message::LoadedMessage::from_message(msg, &mut task_buffer), None, dict);
                for line in task_buffer {
                    tasks.send(line);
                }
            },
            LoadingState::Unloaded => {
                tasks.send(Task::GetMessagesAfter(self.id.clone(), self.labels.front().map(|x| x.id).unwrap_or(MessageId(0)))).expect("could not send!");
                tasks.send(Task::Kick(Response::Message(msg), Duration::from_millis(100))).expect("could not send!");
            },
            LoadingState::Loading => {
                tasks.send(Task::Kick(Response::Message(msg), Duration::from_millis(100))).expect("could not send!");
            },
        }
    }
    pub fn add_complete(&mut self, msg: LoadedMessage, pos: Option<usize>, dict: &mut UserDict) {
        dict.contents
            .entry(msg.user)
            .or_insert_with(|| UserInfo {
                name: msg.username.clone(),
                color: Color::new(),
            });
        let content = msg.content.clone();
        let name = msg.username.clone();
        if let Some(pos) = pos {
            if pos <= self.current {
                self.current += 1;
            }
            if pos <= self.selected {
                self.selected += 1;
            }
            self.labels.insert(pos, msg);
        } else {
            self.labels.push_back(msg);
            if self.labels.len() > 2 && self.labels.len() - 2 == self.current {
                self.current += 1;
            }
        }
        self.flag = true;
    }
    /*
    pub fn add(&mut self, msg: Message, pos: Option<usize>, dict: &mut UserDict, tasks: &Sender<Task>) {
        dict.contents
            .entry(msg.author.id)
            .or_insert_with(|| UserInfo {
                name: msg.author.name.clone(),
                color: Color::new(),
            });
        let content = msg.content.clone();
        let name = msg.author.id;
        if let Some(pos) = pos {
            if pos <= self.current {
                self.current += 1;
            }
            if pos <= self.selected {
                self.selected += 1;
            }
            let mut message = LoadedMessage::from_content(
                name,
                content.lines().map(|x| x.to_string()).collect(),
                msg.timestamp.with_timezone(&Local),
                msg.id,
                msg.author.name,
            );
            for embed in msg.embeds {
                crate::file::add_on("debug", &format!("{:?}", embed));
                message = message.embed(embed);
            }
            for attachment in msg.attachments {
                message = message.attachment(attachment, tasks);
            }
            self.labels.insert(pos, message);
        } else {
            let mut message = LoadedMessage::from_content(
                name,
                content.split('\n').map(|x| x.to_string()).collect(),
                msg.timestamp.with_timezone(&Local),
                msg.id,
                msg.author.name,
            );
            for embed in msg.embeds {
                crate::file::add_on("debug", &format!("{:?}", embed));
                message = message.embed(embed);
            }
            for attachment in msg.attachments {
                message = message.attachment(attachment, tasks);
            }
            self.labels.push_back(message);
            if self.labels.len() > 2 && self.labels.len() - 2 == self.current {
                self.current += 1;
            }
        }
        self.flag = true;
    }*/
    fn remove(&mut self, pos: usize) {
        if pos <= self.current {
            self.current -= 1;
        }
        if pos <= self.selected {
            self.selected -= 1;
        }
        self.labels.remove(pos);
        self.flag = true;
    }
    pub fn draw(&mut self, grid: &Grid, out: &mut Stdout, dict: &mut UserDict, tasks: &Sender<Task>) -> bool {
        if self.flag {
            self.draw_real(grid, out, dict, tasks);
            self.flag = false;
            true
        } else {
            false
        }
    }
    fn draw_real(&mut self, grid: &Grid, out: &mut Stdout, dict: &mut UserDict, tasks: &Sender<Task>) {
        let mut counter = 0;
        let start = self.beginning_pos(grid.height());
        let sample = " ".graphemes(true).cycle();
        let mut hover_pos = usize::MAX;
        let mut selected_pos: usize = usize::MAX; // will never be encountered if not assigned to
        let mut result: Vec<String> = Vec::new(); // contains all the right strings
        for i in start..start + grid.height().min(self.labels.len()) {
            let mut indicator = 0;
            let temp = &self.labels[i].user(dict, grid.len_messages());
            let val = Some(temp).into_iter();
            let temp = self.labels[i].content.attachments.iter();
            for mut j in val.chain(self.labels[i].content.content.iter()).chain(temp) {
                let temp = " ".to_string();
                if j.is_empty() {
                    j = &temp;
                }
                for line in j
                    .graphemes(true)
                    .collect::<Vec<_>>()
                    .chunks(grid.len_messages())
                    .map(|x| x.iter().fold(String::new(), |x, y| x + *y))
                {
                    if i == self.current && indicator == self.current_in_message {
                        hover_pos = counter;
                    }
                    if i == self.selected && indicator == 0 {
                        selected_pos = counter;
                    }
                    counter += 1;
                    indicator += 1;
                    result.push(
                        line.graphemes(true)
                            .chain(sample.clone())
                            .take(grid.len_messages())
                            .collect(),
                    );
                }
            }
            if i == self.current && hover_pos == usize::MAX {
                hover_pos = counter - 1;
            }
        }
        let start = self.beginning_minmax(grid.height(), hover_pos, result.len());
        for i in start..start + grid.height() {
            let val: String = result
                .get(i)
                .unwrap_or(&String::new())
                .graphemes(true)
                .chain(sample.clone())
                .take(grid.len_messages())
                .collect();
            let true_pos = i - start + grid.start_y;
            let _ = queue!(
                out,
                crossterm::cursor::MoveTo(grid.border_3 as u16, true_pos as u16,)
            );
            if i == hover_pos {
                let _ = queue!(
                    out,
                    Print(if grid.messages_selected() {
                        ansi::BACKGROUND_LIGHT_GREY.to_string()
                    } else {
                        ansi::BACKGROUND_GREY.to_string()
                    })
                );
            }
            if i >= selected_pos && i < selected_pos + self.count(grid, self.selected) {
                let _ = queue!(out, Print(ansi::HIGH_INTENSITY.to_string()));
            }
            let _ = queue!(out, Print(val));
            let _ = queue!(
                out,
                crossterm::cursor::MoveTo(grid.border_3 as u16, true_pos as u16,)
            );
            let _ = queue!(out, Print(crate::ansi::RESET.to_string()));
        }
        if self.current < grid.height() / 2 {
            self.update(tasks);
        }
    }
    /// Provides an extra update towards the beginning
    fn update(&mut self, tasks: &Sender<Task>) {
        if matches!(self.before, LoadingState::Unloaded) {
            let id = self.labels[0].id;
            let channel = self.id.clone();
            tasks.send(Task::GetMessagesBefore(channel, id)).expect("Could not send!");
            self.before = LoadingState::Loading;
        }
    }
    pub fn receive_update(&mut self, dict: &mut UserDict, payload: Vec<LoadedMessage>) {
        if payload.is_empty() {
            self.before = LoadingState::Finished;
        } else {
            self.before = LoadingState::Unloaded;
            for msg in payload {
                self.add_complete(msg, Some(0), dict);
            }
        }
    }
    pub fn receive_update_after(&mut self, dict: &mut UserDict, payload: Vec<LoadedMessage>) {
        if payload.is_empty() {
            self.after = LoadingState::Finished;
        } else {
            self.after = LoadingState::Unloaded;
            for msg in payload {
                self.add_complete(msg, None, dict);
            }
        }
    }
    fn beginning_pos(&self, height: usize) -> usize {
        if height >= self.labels.len() || self.current <= height / 2 {
            0
        } else if self.current + height / 2 >= self.labels.len() {
            self.labels.len() - height
        } else {
            self.current - height / 2
        }
    }
    fn beginning_minmax(&self, height: usize, current: usize, len: usize) -> usize {
        if height >= len || current <= height / 2 {
            0
        } else if current + height / 2 >= len {
            len - height
        } else {
            current - height / 2
        }
    }
    fn count(&self, grid: &Grid, pos: usize) -> usize {
        let len = grid.len_messages();
        let mut result = 1; //for the username
        for line in &self.labels[pos].content.content {
            if line.is_empty() {
                result += 1;
            } else {
                result += (line.len() - 1) / len + 1;
            }
        }
        result += self.labels[pos].content.attachments.len();
        result
    }
    pub fn attachment_pos(&self, grid: &Grid) -> Option<usize> {
        let len = self.labels[self.current].content.attachment_url.len();
        if len == 0 {
            None
        } else {
            Some(
                (self.current_in_message + len)
                    .checked_sub(self.count(grid, self.current))
                    .unwrap_or(0)
                    .min(len - 1),
            )
        }
    }
    pub fn message_person(&self, client: &mut Client) -> serenity::Result<PrivateChannel> {
        let u_id = self.labels[self.current].user;
        block_on(u_id.create_dm_channel(Arc::clone(&client.cache_and_http)))
    }
    pub fn open(&self, options: &ExtConfig, grid: &Grid) {
        if let Some(val) = self.attachment_pos(grid) {
            options.open(&self.labels[self.current].content.attachment_url[val]);
        } else {
            panic!("NO FILES!")
        }
    }
}
