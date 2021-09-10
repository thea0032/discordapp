use std::{sync::mpsc::{Receiver, Sender, channel}, thread::sleep, time::Duration};

use futures::future::join_all;
use serenity::model::channel::{Attachment, Channel, Message};

use crate::{block_on::block_on, file::{fs_write, fs_write_2}, message::LoadedMessage};

pub enum Task {
    Download(Attachment, String),
    GetMessages(Channel)
}
impl Task {
    pub async fn execute(self) -> Option<Product> {
        match self {
            Task::Download(attachment, location) => {
                let file = attachment.download().await.unwrap_or(b"Could not find file!".to_vec());
                fs_write_2(file, &location);
                None
            }
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
    Messages(Vec<Message>),
}

pub fn start() -> (Sender<Task>, Receiver<Product>){
    let (send, res) = channel(); 
    let (res2, recv) = channel();
    std::thread::spawn(|| task_init(recv, send));
    (res2, res)
}
fn task_init(recv: Receiver<Task>, send: Sender<Product>) -> Option<()> {
    loop {
        let mut res = vec![recv.recv().ok()?];
        sleep(Duration::from_millis(500));
        for line in recv.try_iter() {
            res.push(line);
        }
        let mut queue = Vec::new();
        for line in res {
            queue.push(line.execute());
        }
        let final_future = join_all(queue);
        let final_res = block_on(final_future);
        for line in final_res {
            if let Some(line) = line {
                send.send(line).ok()?;
            }
        }
    }
}