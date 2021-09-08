use crossterm::event::{KeyCode, KeyEvent};

use super::State;

impl super::Parser {
    pub fn parse_none_category(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            KeyCode::Left => {
                self.grid.context = super::Context::Server;
                self.servers.get().flag();
                self.servers.flag();
            }
            KeyCode::Right => {
                self.grid.context = super::Context::Channel;
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
                self.state = State::Message;
            }
            KeyCode::Char('f') => {
                self.state = State::Filter;
            }
            _ => {}
        }
    }
    pub fn parse_visual_categories(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        
    }
}
