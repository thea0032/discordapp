use std::{sync::mpsc::{Receiver, Sender, channel}, thread::sleep, time::Duration};

use futures::future::join_all;
use serenity::{Client, builder::GetMessages, framework::StandardFramework, model::{channel::{Attachment, Channel, Message}, id::{ChannelId, MessageId}}};

use crate::{DummyHandler, block_on::{self, block_on}, file::{fs_write_2}};
pub enum MessageSearch {
    Before(u64),
    After(u64),
    None,
}
pub enum Task {
    Download(Attachment, String),
    GetMessages(Channel, MessageSearch),
    Wait,
    Kill,
}
impl Task {
    pub async fn execute(self, client: &Client) -> Option<Product> {
        match self {
            Task::Download(attachment, location) => {
                let file = attachment.download().await.unwrap_or(b"Could not find file!".to_vec());
                fs_write_2(file, &location);
                None
            }
            Task::GetMessages(channel, search) => {
                match channel.clone() {
                    Channel::Guild(ch) => {
                        let v = ch.messages(client.cache_and_http.http.clone(), |x| match search {
                            MessageSearch::Before(msg) => x.before(msg),
                            MessageSearch::After(msg) => x.after(msg),
                            MessageSearch::None => x,
                        }).await.unwrap_or(vec![]);
                        Some(Product::Messages(v, channel.id()))
                    },
                    Channel::Private(ch) => {
                        let v = ch.messages(client.cache_and_http.http.clone(), |x| match search {
                            MessageSearch::Before(msg) => x.before(msg),
                            MessageSearch::After(msg) => x.after(msg),
                            MessageSearch::None => x,
                        }).await.unwrap_or(vec![]);
                        Some(Product::Messages(v, channel.id()))
                    }
                    Channel::Category(_) => panic!("Cannot get messages from a category!"),
                    _ => panic!()
                }
            }
            _ => None
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

pub enum Product {
    Messages(Vec<Message>, ChannelId),
    CanSave,
    Killed,
}
pub fn start(token: &str) -> (Sender<Task>, Receiver<Product>){
    let (send, res) = channel(); 
    let (res2, recv) = channel();
    let framework = StandardFramework::new();
    let client = block_on::block_on(Client::builder(&token)
        .framework(framework)
        .event_handler(DummyHandler))
        .expect("Error creating client");
    std::thread::spawn(|| task_init(recv, send, client));
    (res2, res)
}
fn task_init(recv: Receiver<Task>, send: Sender<Product>, client: Client) -> Option<()> {
    'outer: loop {
        let mut res = vec![recv.recv().ok()?];
        sleep(Duration::from_millis(500));
        for line in recv.try_iter() {
            res.push(line);
        }
        let mut queue = Vec::new();
        for line in res {
            queue.push(line.execute(&client));
        }
        let final_future = join_all(queue);
        let final_res = block_on(final_future);
        for line in final_res {
            if let Some(line) = line {
                let kill = matches!(line, Product::Killed);
                send.send(line).ok()?;
                if kill {
                    break 'outer;
                }
            }
        }
    }
    Some(())
}