use std::io::Stdout;

use crossterm::{queue, style::Print};
use serenity::model::{channel::{GuildChannel}, id::GuildId};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    ansi,
    channels::{self, Channels},
    grid::Grid,
};
use crate::servers::Unread;
pub struct Categories {
    pub labels: Vec<String>,
    pub unread: Vec<Unread>,
    pub contents: Vec<Channels>,
    pub current: usize,
    pub selected: usize,
    pub s_id: Option<GuildId>,
    flag: bool,
}
impl Categories {
    pub fn new(catch_name: &str, server: Option<GuildId>) -> Self {
        Self {
            labels: vec![catch_name.to_string()],
            unread: vec![Unread::Read],
            contents: vec![channels::Channels::new(None)],
            current: 0,
            selected: 0,
            s_id: server,
            flag: true,
        }
    }
    pub fn flag(&mut self) {
        self.flag = true;
    }
    pub fn up(&mut self) -> usize {
        if self.current > 0 {
            self.current -= 1;
            self.flag = true;
        }
        self.current
    }
    pub fn down(&mut self) -> usize {
        if self.current < self.labels.len() - 1 {
            self.current += 1;
            self.flag = true;
        }
        self.current
    }
    pub fn select(&mut self) {
        self.selected = self.current;
        self.flag = true;
    }
    pub fn mark(&mut self, pos: usize, state: Unread) {
        self.unread[pos] = state;
        self.flag = true;
    }
    pub fn add(&mut self, name: String, pos: Option<usize>, ch: Option<GuildChannel>) {
        if let Some(pos) = pos {
            if pos < self.current {
                self.current += 1;
            }
            self.unread.insert(pos, Unread::Read);
            self.labels.insert(pos, name);
            self.contents.insert(pos, Channels::new(ch));
        } else {
            self.unread.push(Unread::Read);
            self.labels.push(name);
            self.contents.push(Channels::new(ch));
        }
        self.flag = true;
    }
    pub fn remove(&mut self, pos: usize) {
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
    pub fn get(&mut self) -> &mut Channels {
        &mut self.contents[self.selected]
    }
    pub fn grab(&mut self, spec: usize) -> &mut Channels {
        &mut self.contents[spec]
    }
    pub fn last(&mut self) -> &mut Channels {
        self.contents.last_mut().expect("safe unwrap")
    }
    pub fn switch(&mut self, spec: usize) -> &mut Channels {
        self.selected = spec;
        self.get()
    }
    pub fn draw(&mut self, grid: &Grid, out: &mut Stdout) -> bool {
        if self.flag {
            self.draw_real(grid, out);
            self.flag = false;
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
                .unwrap_or(&String::new())
                .graphemes(true)
                .chain(sample.clone())
                .take(grid.len_categories() - 4)
                .collect();
            let true_pos = i - start + grid.start_y;
            let _ = queue!(
                out,
                crossterm::cursor::MoveTo(grid.border_1 as u16, true_pos as u16,)
            );
            if i == self.current {
                let _ = queue!(
                    out,
                    Print(if grid.categories_selected() {
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
}
