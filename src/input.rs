mod categories;
mod channels;
mod messages;
mod servers;

use std::{
    collections::HashMap,
    io::{stdout, Stdout},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{spawn, JoinHandle},
    time::Duration,
};

use crate::{block_on::block_on, download::{Product, Task}, file::{FileOptions, get_str}, grid::Grid, message::UserDict, save::{Autosave, ParserSave, Return, load}, servers::Servers, textbox::Textbox};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{Clear, ClearType},
};
use serde_json::json;
use serenity::{
    client,
    http::{GuildPagination, Http},
    model::{
        channel::{Channel, ChannelType, GuildChannel, Message, PrivateChannel},
        guild::GuildInfo,
        id::{ChannelId, GuildId},
    },
    Client,
};

fn grab(out: Sender<Event>) {
    spawn(move || loop {
        if let Ok(val) = read() {
            out.send(val).expect("A critical error occurred: ");
        }
    });
}
#[derive(PartialEq, Eq)]
pub enum State {
    None,
    Message,
    Filter,
    Quit,
    Visual,
}
pub enum Context {
    Server,
    Category,
    Channel,
    Message,
}
pub enum Response {
    Message(client::Context, Message),
}
pub struct Parser {
    pub input_server: Receiver<Response>,
    pub client: Client,
    pub input_user: Receiver<Event>,
    pub servers: Servers,
    pub state: State,
    pub grid: Grid,
    pub out: Stdout,
    pub message_box: Textbox,
    pub temp_box: Textbox,
    pub user_dict: UserDict,
    pub file_options: FileOptions,
    pub autosave: Autosave,
    pub tasks: Sender<Task>,
    pub products: Receiver<Product>
}
impl Parser {
    pub fn new(input_server: Receiver<Response>, client: Client) -> Parser {
        match load(input_server, client) {
            Ok(val) => val,
            Err(Return(why, input_server, client)) => {
                get_str(&(why + " press enter to load a new save, or ctrl+c to exit and try again."));
                Self::complete_new(input_server, client)
            },
        }
    }
    pub fn from_save(save: ParserSave, input_server: Receiver<Response>, client: Client) -> Parser {
        let (temp, input_user) = channel();
        grab(temp);
        let (max_x, max_y) = crossterm::terminal::size().expect("Cannot read size of terminal");
        let max_x = max_x as usize;
        let max_y = max_y as usize;
        let grid = Grid::new(max_x, max_y);
        let (tasks, products) = crate::download::start();
        Parser {
            input_server,
            client,
            state: State::None,
            servers: save.servers.reload(),
            input_user,
            grid,
            out: stdout(),
            message_box: Textbox::new(max_x),
            temp_box: Textbox::new(max_x),
            user_dict: save.user_dict,
            file_options: save.file_options,
            autosave: Autosave::new(),
            tasks,
            products,
        }
    }
    pub fn complete_new(input_server: Receiver<Response>, client: Client) -> Parser {
        let (temp, input_user) = channel();
        grab(temp);
        let (max_x, max_y) = crossterm::terminal::size().expect("Cannot read size of terminal");
        let (tasks, products) = crate::download::start();
        let mut parser = Parser {
            input_server,
            client,
            state: State::None,
            servers: Servers::new(),
            input_user,
            grid: Grid::new(max_x as usize, max_y as usize),
            out: stdout(),
            message_box: Textbox::new(max_x as usize),
            temp_box: Textbox::new(max_x as usize),
            user_dict: UserDict::new(),
            file_options: FileOptions::new(),
            autosave: Autosave::new(),
            tasks,
            products,
        };
        parser.network_update_first();
        parser
    }
    pub fn start(self) -> JoinHandle<Self> {
        std::thread::spawn(move || self.start_real())
    }
    pub fn start_real(mut self) -> Self {
        'outer: loop {
            while let Ok(val) = self.input_server.try_recv() {
                match val {
                    Response::Message(_, message) => {
                        self.add_message(message);
                    }
                }
            }
            while let Ok(val) = self.input_user.recv_timeout(Duration::from_millis(50)) {
                match val {
                    Event::Key(key) => match self.state {
                        State::None => self.parse_none(key),
                        State::Filter => {}
                        State::Message => self.parse_message(key),
                        State::Quit => {
                            if self.parse_quit(key) {
                                // this unwrap is safe
                                break 'outer;
                            }
                        }
                        State::Visual => self.parse_visual(key),
                    },
                    Event::Mouse(_) => todo!(),
                    Event::Resize(length, height) => {
                        self.grid.update(
                            self.message_box.lines().min(self.grid.max_box_len),
                            height as usize,
                            length as usize,
                        );
                        self.reset_all();
                    }
                }
            }
            if self.state != State::Quit {
                self.draw();
            }
            self.try_save();
        }
        self.force_save();
        self
    }
    pub fn try_save(&mut self) {
        if self.autosave.should_save() {
            self.autosave = Autosave::save(self);
        }
    }
    pub fn force_save(&mut self) {
        self.autosave = Autosave::save(self);
    }
    fn add_message(&mut self, message: Message) {
        let channel = message.channel_id;
        if let Some(server) = self
            .servers
            .contents
            .iter()
            .position(|x| x.s_id == message.guild_id)
        {
            let grabbed = self.servers.grab(server);
            let mut category = None;
            let mut ch = None;
            for (i, mut item) in grabbed
                .contents
                .iter()
                .map(|x| x.contents.iter())
                .enumerate()
            {
                if let Some(val) = item.position(|x| {
                    x.id()
                        .and_then(|x| Some(x.id() == channel))
                        .unwrap_or(false)
                }) {
                    category = Some(i);
                    ch = Some(val);
                }
            }
            if let (Some(category), Some(channel)) = (category, ch) {
                if !self
                    .servers
                    .grab3(server, category, channel)
                    .update_to_end(&mut self.client, &mut self.user_dict, &self.tasks)
                {
                    self.servers
                        .grab3(server, category, channel)
                        .assume_loaded()
                        .add(message, None, &mut self.user_dict, &self.tasks);
                }
            } else {
                panic!("No channel/category found!");
            }
        } else {
            panic!("No server found!");
        }
    }
    fn parse_none(&mut self, input: KeyEvent) {
        match self.grid.context {
            Context::Server => self.parse_none_server(input),
            Context::Category => self.parse_none_category(input),
            Context::Channel => self.parse_none_channel(input),
            Context::Message => self.parse_none_message(input),
        }
    }
    fn parse_message(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            KeyCode::Backspace => {
                self.message_box.backspace();
            }
            KeyCode::Enter => {
                let message = &json!({"content": self.message_box.flush()});
                self.grid.update_msg(1);
                if let Some(val) = self.servers.get3().id().and_then(|x| x.guild()) {
                    if let Err(why) = block_on(self.http().send_message(val.id.0, message)) {
                        let v = why.to_string();
                        self.temp_box.flush();
                        self.temp_box.add_to_end(vec![v]);
                        let temp = (self.grid.end_y - self.temp_box.lines()) as u16;
                        self.temp_box.draw(0, temp, &mut self.out, false);
                    }
                }
                if let Some(val) = self.servers.get3().id().and_then(|x| x.private()) {
                    if let Err(why) = block_on(self.http().send_message(val.id.0, message)) {
                        let v = why.to_string();
                        self.temp_box.flush();
                        self.temp_box.add_to_end(vec![v]);
                        let temp = (self.grid.end_y - self.temp_box.lines()) as u16;
                        self.temp_box.draw(0, temp, &mut self.out, false);
                    }
                }
                self.reset_all();
            }
            KeyCode::Left => self.message_box.left(),
            KeyCode::Right => self.message_box.right(),
            KeyCode::Up => self.message_box.up(),
            KeyCode::Down => self.message_box.down(),
            KeyCode::Delete => {
                self.message_box.delete();
            }
            KeyCode::Char(val) => {
                self.message_box.add_char(val);
            }
            KeyCode::Esc => {
                self.state = State::None;
            }
            KeyCode::Tab => {
                self.message_box.newline();
            }
            _ => todo!(),
        }
    }
    fn draw(&mut self) {
        if self.message_box.flag {
            let prev = self.grid.border_y;
            self.grid.border_y =
                self.grid.end_y - self.message_box.lines().min(self.grid.max_box_len).max(1);
            if self.grid.border_y != prev {
                self.flag_all();
            }
        }
        self.servers.draw(&self.grid, &mut self.out);
        self.servers.get().draw(&self.grid, &mut self.out);
        self.servers.get2().draw(&self.grid, &mut self.out);
        self.servers
            .get3()
            .draw(&self.grid, &mut self.out, &mut self.user_dict, &mut self.client, &self.tasks);
        self.message_box.draw(
            self.grid.start_x as u16,
            self.grid.border_y as u16,
            &mut self.out,
            true,
        );
        let _ = execute!(self.out);
    }
    fn parse_quit_start(&mut self) {
        self.temp_box = Textbox::new(self.grid.total_across());
        self.temp_box
            .add_to_end(vec!["Are you sure you want to quit?".to_string()]);
        self.temp_box.draw(
            self.grid.start_x as u16,
            self.grid.border_y as u16,
            &mut self.out,
            true,
        );
        let _ = execute!(self.out);
    }
    fn parse_quit(&mut self, input: KeyEvent) -> bool {
        match input.code {
            KeyCode::Char('Y') | KeyCode::Char('y') | KeyCode::Char('C') | KeyCode::Char('c') => {
                true
            }
            _ => {
                self.state = State::None;
                self.temp_box.flush();
                self.temp_box.draw(
                    self.grid.start_x as u16,
                    self.grid.border_y as u16,
                    &mut self.out,
                    true,
                );
                let _ = queue!(self.out, Clear(ClearType::CurrentLine));
                self.message_box.draw(
                    self.grid.start_x as u16,
                    self.grid.border_y as u16,
                    &mut self.out,
                    true,
                );
                let _ = execute!(self.out);
                false
            }
        }
    }
    fn reset_all(&mut self) {
        self.servers.flag();
        self.servers.get().flag();
        self.servers.get2().flag();
        self.servers.get3().flag();
        self.message_box.flag();
        self.draw();
    }
    fn flag_all(&mut self) {
        self.servers.flag();
        self.servers.get().flag();
        self.servers.get2().flag();
        self.servers.get3().flag();
        self.message_box.flag();
    }
    fn http(&self) -> Arc<Http> {
        Arc::clone(&self.client.cache_and_http.http)
    }
    fn network_update_first(&mut self) {
        let v = block_on(
            self.http()
                .get_guilds(&GuildPagination::After(GuildId(0)), 100),
        );
        if let Ok(val) = v {
            let result = self.network_update_channels(val.clone());
            for (item, server) in result.into_iter().zip(val) {
                // each server
                self.servers.add(server.name, None, server.id);
                for (category, item) in item {
                    // each category
                    if let Some(_) = category {
                        self.servers.last().add(
                            item.category
                                .clone()
                                .and_then(|x| Some(x.name))
                                .unwrap_or("unnamed category".to_string()),
                            None,
                            item.category.clone(),
                        );
                        for channel in item.channels {
                            // each channel
                            self.servers.last2().add(
                                channel.name.clone(),
                                None,
                                Channel::Guild(channel),
                            );
                        }
                    } else {
                        for channel in item.channels {
                            // each channel
                            self.servers.last().grab(0).add(
                                channel.name.clone(),
                                None,
                                Channel::Guild(channel),
                            );
                        }
                    }
                }
            }
        }
        self.load_dms();
    }
    fn network_update_channels(
        &mut self,
        servers: Vec<GuildInfo>,
    ) -> Vec<HashMap<Option<ChannelId>, Category>> {
        let http = self.http().clone();
        let mut collection: HashMap<GuildId, HashMap<Option<ChannelId>, Category>> = HashMap::new();
        for line in &servers {
            let v = block_on(http.get_channels(line.id.0)).unwrap_or(Vec::new());
            let mut temp: HashMap<Option<ChannelId>, Category> = HashMap::new();
            for line in v {
                match line.kind {
                    ChannelType::Text => {
                        temp.entry(line.category_id)
                            .or_insert_with(Category::new)
                            .channels
                            .push(line);
                    }
                    ChannelType::Category => {
                        temp.entry(Some(line.id))
                            .or_insert_with(Category::new)
                            .category = Some(line.clone());
                    }
                    _ => {}
                }
            }
            collection.insert(line.id.clone(), temp);
        }
        let mut vals: Vec<Option<HashMap<Option<ChannelId>, Category>>> = vec![None; servers.len()];
        for line in collection {
            let pos = servers
                .iter()
                .position(|x| x.id == line.0)
                .expect("safe unwrap");
            vals[pos] = Some(line.1);
        }
        vals.into_iter().map(|x| x.expect("safe unwrap")).collect()
    }
    pub fn get_messages(client: &Client, id: GuildChannel) -> (Vec<Message>, bool) {
        let v = block_on(client.cache_and_http.http.get_messages(id.id.0, "")).unwrap();
        let result = v.len() == 50;
        (v, result)
    }
    pub fn get_messages_p(client: &Client, id: PrivateChannel) -> (Vec<Message>, bool) {
        let v = block_on(client.cache_and_http.http.get_messages(id.id.0, "")).unwrap();
        let result = v.len() == 50;
        (v, result)
    }
    pub fn load_dms(&mut self) {
        let dms = block_on(self.http().get_user_dm_channels()).expect("WHY?!!");
        for dm in dms {
            self.servers.grab2(0, 0).add(
                dm.name().split(' ').skip(2).next().unwrap().to_string(),
                None,
                Channel::Private(dm.clone()),
            );
        }
    }
    pub fn message_person(&mut self) {
        if let Ok(val) = self
            .servers
            .get3()
            .assume_loaded()
            .message_person(&mut self.client)
        {
            let pos = self
                .servers
                .grab2(0, 0)
                .contents
                .iter_mut()
                .position(|x| x.id().and_then(|x| Some(x.id() == val.id)).unwrap_or(false));
            if let Some(val) = pos {
                self.servers.switch3(0, 0, val);
            } else {
                self.servers
                    .grab2(0, 0)
                    .add(val.name(), None, Channel::Private(val));
            }
        }
    }
    pub fn parse_visual(&mut self, event: KeyEvent) {
        match self.grid.context {
            Context::Server => self.parse_visual_servers(event),
            Context::Category => self.parse_visual_categories(event),
            Context::Channel => self.parse_visual_channels(event),
            Context::Message => self.parse_visual_messages(event),
        }
    }
}
#[derive(Clone)]
struct Category {
    channels: Vec<GuildChannel>,
    category: Option<GuildChannel>,
}
impl Category {
    pub fn new() -> Category {
        Category {
            channels: Vec::new(),
            category: None,
        }
    }
}
