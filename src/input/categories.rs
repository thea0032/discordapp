use crossterm::event::{KeyCode, KeyEvent};

use super::State;

impl super::Parser {
    pub fn parse_none_category(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            KeyCode::Left => {
                self.int.grid.context = super::Context::Server;
                self.servers.get().flag();
                self.servers.flag();
            }
            KeyCode::Right => {
                self.int.grid.context = super::Context::Channel;
                self.servers.get().flag();
                self.servers.get2().flag();
            }
            KeyCode::Up => {
                self.servers.get().up();
            }
            KeyCode::Down => {
                self.servers.get().down();
            }
            KeyCode::Enter => {
                self.servers.get().select();
                self.reset_all();
            }
            KeyCode::Char('t') => {
                self.int.state = State::Message;
            }
            KeyCode::Char('f') => {
                self.int.state = State::Filter;
            }
            _ => {}
        }
    }
    pub fn parse_visual_categories(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            KeyCode::Backspace | KeyCode::Delete | KeyCode::Esc => self.int.state = State::None,
            KeyCode::Left => {
                self.int.grid.context = super::Context::Server;
                self.servers.get().flag();
                self.servers.flag();
            }
            KeyCode::Right => {
                self.int.grid.context = super::Context::Channel;
                self.servers.get().flag();
                self.servers.get2().flag();
            }
            KeyCode::Up => {
                self.servers.get().up();
            }
            KeyCode::Down => {
                self.servers.get().down();
            }
            KeyCode::Enter => {
                self.servers.get().select();
                self.reset_all();
            }
            KeyCode::Char('c') => {
                self.servers.get().color();
            },
            _ => {}
        }
    }
}
