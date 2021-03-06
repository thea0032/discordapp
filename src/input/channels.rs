use crossterm::event::{KeyCode, KeyEvent};

use super::State;

impl super::Parser {
    pub fn parse_none_channel(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            KeyCode::Left => {
                self.int.grid.context = super::Context::Category;
                self.reset_all();
            }
            KeyCode::Right => {
                self.int.grid.context = super::Context::Message;
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
                self.int.state = State::Message;
            }
            _ => {}
        }
    }
    pub fn parse_visual_channels(&mut self, input: KeyEvent) {
        let KeyEvent { code, ..}: KeyEvent = input;
        match code {
            KeyCode::Left => {
                self.int.grid.context = super::Context::Category;
                self.reset_all();
            }
            KeyCode::Right => {
                self.int.grid.context = super::Context::Message;
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
            KeyCode::Char('c') => self.servers.get2().color(),
            KeyCode::Char('s') => self.servers.get2().select_color(),
            KeyCode::Esc | KeyCode::Delete | KeyCode::Backspace => self.int.state = State::None,
            _ => {},
        }
    }
}
