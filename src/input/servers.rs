use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};

use super::State;

impl super::Parser {
    pub fn parse_none_server(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            KeyCode::Right => {
                self.grid.context = super::Context::Category;
                self.servers.flag();
                self.servers.get().flag();
            }
            KeyCode::Up => {
                self.servers.up();
            }
            KeyCode::Down => {
                self.servers.down();
            }
            KeyCode::Enter => {
                self.servers.select();
                self.reset_all();
            }
            KeyCode::Char('t') => {
                self.state = State::Message;
            }
            KeyCode::Char('q') => {
                self.state = State::Quit;
                self.parse_quit_start();
            }
            _ => {}
        }
    }
}