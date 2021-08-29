use std::env::current_dir;
use std::fs;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::ClearType;
use input::Response;
use serenity::framework::StandardFramework;
use serenity::{
    async_trait,
    model::{channel::Message},
    prelude::*,
};

use crate::file::from_relative_path;
use crate::file::get_file;
use crate::file::get_file_root;
use crate::file::open_with;
use crate::file::run;

#[allow(dead_code)]
mod ansi;
pub mod categories;
pub mod channels;
mod event;
pub mod grid;
mod input;
pub mod messages;
mod servers;
mod textbox;
mod file;
mod message;
mod format;
mod save;
struct DummyHandler;
impl EventHandler for DummyHandler {

}

struct Handler {
    send: Mutex<Sender<Response>>,
}
#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        
        let sent = Mutex::lock(&self.send).await;
        sent.send(Response::Message(ctx, msg)).expect("the receiver has hung up!");
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
}
fn get_token() -> String {
    if let Ok(val) = fs::read_to_string("token.ignore") {
        val
    } else {
        file::get_str()
    }
}
#[tokio::main]
async fn main() {
    let token = get_token();
    // Configure the client with your Discord bot token in the environment.
    let (send, recv) = channel();
    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let framework = StandardFramework::new();
    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler {send: Mutex::new(send)})
        .await
        .expect("Err creating client");
    let framework = StandardFramework::new();
    let client2 = Client::builder(&token)
        .framework(framework)
        .event_handler(DummyHandler)
        .await
        .expect("Err creating client");
    
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    let future = std::thread::spawn(move || futures::executor::block_on(client.start()));
    let _ = execute!(stdout(), crossterm::terminal::Clear(ClearType::All)).expect("fatal error: "); // clears the terminal
    let parser = input::Parser::new(recv, client2);
    enable_raw_mode().expect("fatal error: ");
    let v = parser.start(); // starts on a new thread
    v.join().expect("fatal error: ");
    disable_raw_mode().expect("fatal error: ");
}