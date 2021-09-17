use std::{task::{RawWaker, Waker}, thread::sleep, time::Duration};
use std::sync::mpsc::{Receiver, Sender, channel};
use futures::{future::join_all, stream::futures_unordered::FuturesUnordered};
use futures::stream::StreamExt;
use futures::stream::Stream;
use serenity::{Client, framework::StandardFramework, model::{channel::{Attachment, Channel, Message}, id::MessageId}};

use crate::{DummyHandler, block_on::{self, block_on}, file::{fs_write_2}, input::Response, message::LoadedMessage, messages::LoadedMessages};
pub enum Control {
    Drain,
    Kill,
}
pub enum Task {
    Download(Attachment, String),
    GetMessagesBefore(Channel, MessageId),
    GetMessagesAfter(Channel, MessageId),
    GetNewMessages(Channel),
    /// Kicks a "can" (response) down the road (waits a duration) until the program is equipped to handle it. 
    Kick(Response, Duration),
}
impl Task {
    pub async fn execute(self, client: &Client) -> (Option<Product>, Vec<Task>) {
        let mut v: Vec<Task> = Vec::new();
        match self {
            Task::Download(attachment, location) => {
                let file = attachment.download().await.unwrap_or(b"Could not find file!".to_vec());
                fs_write_2(file, &location);
                (None, v)
            }
            Task::GetMessagesBefore(channel, search) => {
                match channel.clone() {
                    Channel::Guild(ch) => {
                        let result = ch.messages(client.cache_and_http.http.clone(), |x| x.before(search.0)).await.unwrap_or(vec![]);
                        let result = result.into_iter().map(|x| LoadedMessage::from_message(x, &mut v)).collect::<Vec<_>>();
                        (Some(Product::MessagesBefore(result, channel)), v)
                    },
                    Channel::Private(ch) => {
                        let result = ch.messages(client.cache_and_http.http.clone(), |x| x.before(search.0)).await.unwrap_or(vec![]);
                        let result = result.into_iter().map(|x| LoadedMessage::from_message(x, &mut v)).collect::<Vec<_>>();
                        (Some(Product::MessagesBefore(result, channel)), v)
                    }
                    Channel::Category(_) => panic!("Cannot get messages from a category!"),
                    _ => panic!()
                }
            },
            Task::GetMessagesAfter(channel, search) => {
                match channel.clone() {
                    Channel::Guild(ch) => {
                        let result = ch.messages(client.cache_and_http.http.clone(), |x| x.after(search.0)).await.unwrap_or(vec![]);
                        let result = result.into_iter().map(|x| LoadedMessage::from_message(x, &mut v)).collect::<Vec<_>>();
                        (Some(Product::MessagesBefore(result, channel)), v)
                    },
                    Channel::Private(ch) => {
                        let result = ch.messages(client.cache_and_http.http.clone(), |x| x.after(search.0)).await.unwrap_or(vec![]);
                        let result = result.into_iter().map(|x| LoadedMessage::from_message(x, &mut v)).collect::<Vec<_>>();
                        (Some(Product::MessagesBefore(result, channel)), v)
                    }
                    Channel::Category(_) => panic!("Cannot get messages from a category!"),
                    _ => panic!()
                }
            },
            Task::GetNewMessages(channel) => {
                let result = client.cache_and_http.http.get_messages(channel.id().0, "").await.expect("Could not get messages!");
                let result = result.into_iter().map(|x| LoadedMessage::from_message(x, &mut v)).collect::<Vec<_>>();
                (Some(Product::MessagesNew(result, channel)), v)
            },
            Task::Kick(val, time) => {
                std::thread::sleep(time);
                (Some(Product::Can(val)), v)
            },
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
    MessagesBefore(Vec<LoadedMessage>, Channel),
    MessagesAfter(Vec<LoadedMessage>, Channel),
    MessagesNew(Vec<LoadedMessage>, Channel),
    Can(Response),
    CanSave,
    Killed,
}
pub fn start(token: &str) -> (Sender<Task>, Sender<Control>, Receiver<Product>){
    let (send, res) = channel(); 
    let (res2, recv) = channel();
    let (res3, ctrl) = channel();
    let framework = StandardFramework::new();
    let client = block_on::block_on(Client::builder(&token)
        .framework(framework)
        .event_handler(DummyHandler))
        .expect("Error creating client");
    std::thread::spawn(|| task_init(recv, send, ctrl, client));
    (res2, res3, res)
}

fn task_init(recv: Receiver<Task>, send: Sender<Product>, ctrl: Receiver<Control>, client: Client) -> Option<()> {
    let mut children: Vec<Task> = Vec::new();
    'outer: loop {
        let mut res = Vec::new();
        res.append(&mut children);
        if res.is_empty() {
            sleep(Duration::from_millis(500));
        }
        if let Ok(val) = ctrl.try_recv() {
            match val {
                Control::Drain => send.send(Product::CanSave), // TEMPORARY MEASURE
                Control::Kill => send.send(Product::Killed), // TEMPORARY MEASURE
            }.expect("FUCK YOU");
        }
        for line in recv.try_iter() {
            res.push(line);
        }
        let mut queue = Vec::new();
        for line in res {
            queue.push(line.execute(&client));
        }
        let final_future = join_all(queue);
        let final_res = block_on(final_future);
        for (product, children_spawned) in final_res {
            if let Some(val) = product {
                let should_kill = matches!(val, Product::Killed);
                send.send(val).ok()?;
                if should_kill {
                    break 'outer;
                }
            }
            for line in children_spawned {
                children.push(line);
            }
        }
    }
    Some(())
}
/*
fn task_init(recv: Receiver<Task>, ctrl: Receiver<Control>, send: futures::channel::mpsc::Sender<Product>, client: Client) -> Option<()> {
    while let Ok(task) = recv.recv() {
        let mut send_cloned = send.clone();
        let (children, extra_tasks) = channel();
        tokio::spawn(async {
            let product = task.execute(&client, &children).await;
            if let Some(p) = product {
                send_cloned.start_send(p).expect("Channel closed!");
            }
            
        });
    }
    Some(())
}
*/