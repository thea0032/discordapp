use std::io::Stdout;

use crossterm::{queue, style::Print};
use serenity::model::{channel::{Channel, Message}, id::{ChannelId, GuildId}};
use unicode_segmentation::UnicodeSegmentation;

use crate::{ansi, categories::Categories, channels::Channels, colors::SimpleColor, grid::Grid, messages::Messages};
const DEFAULT: &str = "uncategorized channels";

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Unread {
    Read,
    Unread,
    Mentions(u64),
    Gone,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerLabel {
    name: String,
    color: SimpleColor,
}
impl ServerLabel {
    pub fn new(name: String) -> ServerLabel {
        ServerLabel {
            name,
            color: SimpleColor::new(),
        }
    }
    pub fn to_string(&self) -> String {
        self.color.to_ansi_value() + &self.name 
    }
}
pub struct Servers {
    pub labels: Vec<ServerLabel>,
    pub unread: Vec<Unread>,
    pub contents: Vec<Categories>,
    pub current: usize,
    pub selected: usize,
    pub flag: bool,
}
impl Servers {
    pub fn new() -> Self {
        Self {
            labels: vec![ServerLabel::new("serverless channels".to_string())],
            unread: vec![Unread::Read, Unread::Read],
            contents: vec![Categories::new("DMs", None)],
            current: 0,
            selected: 0,
            flag: true,
        }
    }
    pub fn flag(&mut self) {
        self.flag = true;
    }
    pub fn up(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.flag = true;
        }
    }
    pub fn down(&mut self) {
        if self.current < self.labels.len() - 1 {
            self.current += 1;
            self.flag = true;
        }
    }
    pub fn select(&mut self) {
        self.selected = self.current;
        self.flag = true;
    }
    pub fn mark(&mut self, pos: usize, state: Unread) {
        self.unread[pos] = state;
        self.flag = true;
    }
    pub fn color(&mut self) {
        self.flag = true;
        self.labels[self.current].color.switch_color();
    }
    pub fn add(&mut self, name: String, pos: Option<usize>, id: GuildId) {
        if let Some(pos) = pos {
            if pos < self.current {
                self.current += 1;
            }
            self.unread.insert(pos, Unread::Read);
            self.labels.insert(pos, ServerLabel::new(name));
            self.contents
                .insert(pos, Categories::new(DEFAULT, Some(id)));
        } else {
            self.unread.push(Unread::Read);
            self.labels.push(ServerLabel::new(name));
            self.contents.push(Categories::new(DEFAULT, Some(id)));
        }
        self.flag = true;
    }
    fn remove(&mut self, pos: usize) {
        if pos <= self.current {
            self.current -= 1;
        }
        if pos <= self.selected {
            self.selected -= 1;
        }
        self.unread.remove(pos);
        self.labels.remove(pos);
        self.contents.remove(pos);
        self.flag = true;
    }
    pub fn get(&mut self) -> &mut Categories {
        &mut self.contents[self.selected]
    }
    pub fn get2(&mut self) -> &mut Channels {
        self.get().get()
    }
    pub fn get3(&mut self) -> &mut Messages {
        self.get().get().get()
    }
    pub fn grab(&mut self, spec: usize) -> &mut Categories {
        &mut self.contents[spec]
    }
    pub fn grab2(&mut self, spec: usize, spec2: usize) -> &mut Channels {
        self.grab(spec).grab(spec2)
    }
    pub fn grab3(&mut self, spec: usize, spec2: usize, spec3: usize) -> &mut Messages {
        self.grab(spec).grab(spec2).grab(spec3)
    }
    pub fn last(&mut self) -> &mut Categories {
        self.contents.last_mut().expect("safe unwrap")
    }
    pub fn last2(&mut self) -> &mut Channels {
        self.last().last()
    }
    pub fn last3(&mut self) -> &mut Messages {
        self.last().last().last()
    }
    pub fn switch(&mut self, spec: usize) -> &mut Categories {
        self.selected = spec;
        self.get()
    }
    pub fn switch2(&mut self, spec: usize, spec2: usize) -> &mut Channels {
        self.switch(spec).switch(spec2);
        self.get2()
    }
    pub fn switch3(&mut self, spec: usize, spec2: usize, spec3: usize) -> &mut Messages {
        self.switch(spec).switch(spec2).switch(spec3);
        self.get3()
    }
    pub fn draw(&mut self, grid: &Grid, out: &mut Stdout) -> bool {
        if self.flag {
            self.flag = false;
            self.draw_real(grid, out);
            true
        } else {
            false
        }
    }
    fn draw_real(&mut self, grid: &Grid, out: &mut Stdout) {
        let start = self.beginning_pos(grid.height());
        let sample = " ".graphemes(true).cycle();
        for i in start..start + grid.height() {
            let val: String = self
                .labels
                .get(i)
                .map(|x| x.name.clone())
                .unwrap_or(String::new())
                .graphemes(true)
                .chain(sample.clone())
                .take(grid.len_servers() - 4)
                .collect();
            let true_pos = i - start + grid.start_y;
            let _ = queue!(
                out,
                crossterm::cursor::MoveTo(grid.start_x as u16, true_pos as u16,)
            );
            if i == self.current {
                let _ = queue!(
                    out,
                    Print(if grid.servers_selected() {
                        ansi::BACKGROUND_LIGHT_GREY.to_string()
                    } else {
                        ansi::BACKGROUND_GREY.to_string()
                    })
                );
            }
            if i == self.selected {
                let _ = queue!(out, Print(ansi::HIGH_INTENSITY.to_string()));
            }
            match self.unread.get(i).unwrap_or(&Unread::Read) {
                Unread::Read => {
                    let _ = queue!(out, Print("    ".to_string()));
                }
                Unread::Unread => {
                    let _ = queue!(out, Print("(0) ".to_string()));
                }
                Unread::Mentions(val) => {
                    let _ = queue!(
                        out,
                        Print(format!(
                            "({}) ",
                            if val < &10 {
                                val.to_string()
                            } else {
                                "+".to_string()
                            }
                        ))
                    );
                }
                Unread::Gone => {
                    let _ = queue!(out, Print("!!! ".to_string()));
                }
            };
            let _ = queue!(out, Print(self.labels.get(i).map(|x| x.color.to_ansi_value()).unwrap_or(String::new())));
            let _ = queue!(out, Print(val));
            let _ = queue!(out, Print(crate::ansi::RESET.to_string(),));
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
    pub fn find_channel(&mut self, channel: ChannelId, guild: Option<GuildId>) -> &mut Messages {
        if let Some(server) = self
            .contents
            .iter()
            .position(|x| x.s_id == guild)
        {
            let grabbed = self.grab(server);
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
                return self.grab3(server, category, channel);
            } else {
                panic!("No channel/category found!");
            }
        } else {
            panic!("No server found!");
        }
    }
}
