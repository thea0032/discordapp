use crossterm::event::{KeyCode, KeyEvent};

use super::State;

impl super::Parser {
    pub fn parse_none_channel(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            KeyCode::Left => {
                self.grid.context = super::Context::Category;
                self.reset_all();
            }
            KeyCode::Right => {
                self.grid.context = super::Context::Message;
                self.reset_all();
            }
            KeyCode::Up => {
                self.servers.get2().up();
            }
            KeyCode::Down => {
                self.servers.get2().down();
            }
            KeyCode::Enter => {
                self.servers.get2().select();
                self.reset_all();
            }
            KeyCode::Char('t') => {
                self.state = State::Message;
            }
            _ => {}
        }
    }
}
