mod categories;
mod channels;
mod messages;
mod servers;

use std::{collections::HashMap, io::{stdout, Stdout}, sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    }, thread::{spawn, JoinHandle}, time::Duration};

use crate::{block_on::block_on, file::{FileOptions, get_str}, grid::Grid, message::UserDict, save::{Autosave, ParserSave, Return, load, save}, servers::Servers, task::{Control, Product, Task}, textbox::Textbox};
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
pub const REQUEST_LEN:usize = 50;

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
    Message(Message),
}
pub struct ParserIO {
    pub input_server: Receiver<Response>,
    pub client: Client,
    pub input_user: Receiver<Event>,
    pub out: Stdout,
    pub tasks: Sender<Task>,
    pub controller: Sender<Control>,
    pub products: Receiver<Product>
}
pub struct ParserInternal {
    pub state: State,
    pub grid: Grid,
    pub user_dict: UserDict,
    pub file_options: FileOptions,
    pub autosave: Autosave,
}
pub struct Parser {
    pub io: ParserIO,
    pub int: ParserInternal,
    pub servers: Servers,
    pub message_box: Textbox,
    pub temp_box: Textbox,
}
impl Parser {
    pub fn new(input_server: Receiver<Response>, client: Client, tasks: Sender<Task>, controller: Sender<Control>, products: Receiver<Product>) -> Parser {
        match load(input_server, client, tasks, controller, products) {
            Ok(val) => val,
            Err(Return(why, input_server, client, tasks, controller, products)) => {
                get_str(&(why + " press enter to load a new save, or ctrl+c to exit and try again."));
                Self::complete_new(input_server, client, tasks, controller, products)
            },
        }
    }
    pub fn from_save(save: ParserSave, input_server: Receiver<Response>, client: Client, tasks: Sender<Task>, controller: Sender<Control>, products: Receiver<Product>) -> Parser {
        let (temp, input_user) = channel();
        grab(temp);
        let (max_x, max_y) = crossterm::terminal::size().expect("Cannot read size of terminal");
        let max_x = max_x as usize;
        let max_y = max_y as usize;
        let grid = Grid::new(max_x, max_y);
        Parser {
            io: ParserIO {
                input_server,
                client,
                out: stdout(),
                tasks,
                controller,
                products,
                input_user,
            },
            int: ParserInternal {
                state: State::None,
                grid,
                user_dict: save.user_dict,
                autosave: Autosave::new(),
                file_options: FileOptions::new(),
            },
            servers: save.servers.reload(),
            message_box: Textbox::new(max_x),
            temp_box: Textbox::new(max_x),
        }
    }
    pub fn complete_new(input_server: Receiver<Response>, client: Client, tasks: Sender<Task>, controller: Sender<Control>, products: Receiver<Product>) -> Parser {
        let (temp, input_user) = channel ();
        grab(temp);
        let (max_x, max_y) = crossterm::terminal::size().expect("Cannot read size of terminal");
        let grid = Grid::new(max_x as usize, max_y as usize);
        let mut parser = Parser {
            io: ParserIO {
                input_server,
                client,
                out: stdout(),
                tasks,
                controller,
                products,
                input_user,
            },
            int: ParserInternal {
                state: State::None,
                grid,
                user_dict: UserDict::new(),
                autosave: Autosave::new(),
                file_options: FileOptions::new(),
            },
            servers: Servers::new(),
            message_box: Textbox::new(max_x as usize),
            temp_box: Textbox::new(max_x as usize),
        };
        parser.network_update_first();
        parser
    }
    pub fn start(self) -> JoinHandle<Self> {
        std::thread::spawn(move || self.start_real())
    }
    pub fn start_real(mut self) -> Self {
        'outer: loop {
            std::thread::sleep(Duration::from_millis(10));
            let products: Vec<Product> = self.io.products.try_iter().collect();
            for product in products {
                self.handle_product(product);
            }
            let events:Vec<Event> = self.io.input_user.try_iter().collect();
            for line in events {
                if self.handle_event(line) {
                    break 'outer;
                }
            }
            let responses:Vec<Response> = self.io.input_server.try_iter().collect();
            for line in responses {
                self.handle_response(line);
            }
            if self.int.state != State::Quit {
                self.draw();
            }
            self.save_state();
        }
        self.end_state();
        self
    }
    pub fn handle_response(&mut self, resp: Response) {
        match resp {
            Response::Message(message) => {
                self.add_message(message);
            }
        }
    }
    pub fn handle_event(&mut self, e: Event) -> bool {
        match e {
            Event::Key(key) => match self.int.state {
                State::None => self.parse_none(key),
                State::Filter => {}
                State::Message => self.parse_message(key),
                State::Quit => {
                    if self.parse_quit(key) {
                        return true
                    }
                }
                State::Visual => self.parse_visual(key),
            },
            Event::Mouse(_) => todo!(),
            Event::Resize(length, height) => {
                self.int.grid.update(
                    self.message_box.lines().min(self.int.grid.max_box_len),
                    height as usize,
                    length as usize,
                );
                self.reset_all();
            }
        }
        false
    }
    /// Returns whether a save can happen. 
    pub fn handle_product(&mut self, p: Product) -> bool {
        match p {
            Product::MessagesBefore(content, channel) => {
                let msg = self.servers.find_channel(channel.id(), if let Channel::Guild(v) = &channel {Some(v.guild_id)} else {None});
                msg.assume_loaded().receive_update(&mut self.int.user_dict, content);
            },
            Product::MessagesAfter(content, channel) => {
                let msg = self.servers.find_channel(channel.id(), if let Channel::Guild(v) = &channel {Some(v.guild_id)} else {None});
                msg.assume_loaded().receive_update_after(&mut self.int.user_dict, content);
            },
            Product::MessagesNew(content, channel) => {
                    let msg = self.servers.find_channel(channel.id(), if let Channel::Guild(v) = &channel {Some(v.guild_id)} else {None});;
                    let more_messages: bool = content.len() >= REQUEST_LEN;
                    msg.receive_new(&mut self.int.user_dict, &mut self.io.tasks, content, more_messages);
            },
            Product::CanSave | Product::Killed =>return true,
            Product::Can(val) => self.handle_response(val),
        }
        false
    }
    pub fn save_state(&mut self) {
        if self.int.autosave.should_save() {
            self.io.controller.send(Control::Drain).expect("Failed to mark!");
            'outer: loop  {
                let products: Vec<Product> = self.io.products.try_iter().collect();
                for product in products {
                    if self.handle_product(product) {
                        save(&self).expect("Could not save!");
                        self.int.autosave = Autosave::save(self);
                        break 'outer;
                    }
                }
            }
        }
    }
    pub fn end_state(&mut self) {
        self.io.controller.send(Control::Kill).expect("Failed to mark!");
        'outer: loop {
            let products: Vec<Product> = self.io.products.try_iter().collect();
            for product in products {
                if self.handle_product(product) {
                    save(&self).expect("Could not save!");
                    self.int.autosave = Autosave::save(self);
                    break 'outer;
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }

    }
    fn add_message(&mut self, message: Message) {
        let ch = message.channel_id;
        let guild = message.guild_id;
        let res = self.servers.find_channel(ch, guild);
        res.receive_message(&mut self.int.user_dict, &self.io.tasks, message);
    }
    fn parse_none(&mut self, input: KeyEvent) {
        match self.int.grid.context {
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
                self.int.grid.update_msg(1);
                if let Some(val) = self.servers.get3().id().and_then(|x| x.guild()) {
                    if let Err(why) = block_on(self.http().send_message(val.id.0, message)) {
                        let v = why.to_string();
                        self.temp_box.flush();
                        self.temp_box.add_to_end(vec![v]);
                        let temp = (self.int.grid.end_y - self.temp_box.lines()) as u16;
                        self.temp_box.draw(0, temp, &mut self.io.out, false);
                    }
                }
                if let Some(val) = self.servers.get3().id().and_then(|x| x.private()) {
                    if let Err(why) = block_on(self.http().send_message(val.id.0, message)) {
                        let v = why.to_string();
                        self.temp_box.flush();
                        self.temp_box.add_to_end(vec![v]);
                        let temp = (self.int.grid.end_y - self.temp_box.lines()) as u16;
                        self.temp_box.draw(0, temp, &mut self.io.out, false);
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
                self.int.state = State::None;
            }
            KeyCode::Tab => {
                self.message_box.newline();
            }
            _ => todo!(),
        }
    }
    fn draw(&mut self) {
        if self.message_box.flag {
            let prev = self.int.grid.border_y;
            self.int.grid.border_y =
                self.int.grid.end_y - self.message_box.lines().min(self.int.grid.max_box_len).max(1);
            if self.int.grid.border_y != prev {
                self.flag_all();
            }
        }
        self.servers.draw(&self.int.grid, &mut self.io.out);
        self.servers.get().draw(&self.int.grid, &mut self.io.out);
        self.servers.get2().draw(&self.int.grid, &mut self.io.out);
        self.servers
            .get3()
            .draw(&self.int.grid, &mut self.io.out, &mut self.int.user_dict, &self.io.tasks);
        self.message_box.draw(
            self.int.grid.start_x as u16,
            self.int.grid.border_y as u16,
            &mut self.io.out,
            true,
        );
        let _ = execute!(self.io.out);
    }
    fn parse_quit_start(&mut self) {
        self.temp_box = Textbox::new(self.int.grid.total_across());
        self.temp_box
            .add_to_end(vec!["Are you sure you want to quit?".to_string()]);
        self.temp_box.draw(
            self.int.grid.start_x as u16,
            self.int.grid.border_y as u16,
            &mut self.io.out,
            true,
        );
        let _ = execute!(self.io.out);
    }
    fn parse_quit(&mut self, input: KeyEvent) -> bool {
        match input.code {
            KeyCode::Char('Y') | KeyCode::Char('y') | KeyCode::Char('C') | KeyCode::Char('c') => {
                true
            }
            _ => {
                self.int.state = State::None;
                self.temp_box.flush();
                self.temp_box.draw(
                    self.int.grid.start_x as u16,
                    self.int.grid.border_y as u16,
                    &mut self.io.out,
                    true,
                );
                let _ = queue!(self.io.out, Clear(ClearType::CurrentLine));
                self.message_box.draw(
                    self.int.grid.start_x as u16,
                    self.int.grid.border_y as u16,
                    &mut self.io.out,
                    true,
                );
                let _ = execute!(self.io.out);
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
        Arc::clone(&self.io.client.cache_and_http.http)
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
    
    fn network_update_subsequent(&mut self) {
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
            .message_person(&mut self.io.client)
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
        match self.int.grid.context {
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
