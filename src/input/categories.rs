use crossterm::event::KeyEvent;

use super::State;

impl super::Parser {
    pub fn parse_none_category(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        match code {
            crossterm::event::KeyCode::Left => {
                self.grid.context = super::Context::Server;
                self.servers.get().flag();
                self.servers.flag();
            }
            crossterm::event::KeyCode::Right => {
                self.grid.context = super::Context::Channel;
                self.servers.get().flag();
                self.servers.get2().flag();
            }
            crossterm::event::KeyCode::Up => {
                self.servers.get().up();
            }
            crossterm::event::KeyCode::Down => {
                self.servers.get().down();
            }
            crossterm::event::KeyCode::Enter => {
                self.servers.get().select();
                self.reset_all();
            }
            crossterm::event::KeyCode::Char('t') => {
                self.state = State::Message;
            }
            crossterm::event::KeyCode::Char('f') => {
                self.state = State::Filter;
            }
            _ => {}
        }
    }
}
