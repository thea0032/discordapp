use crossterm::event::{KeyCode, KeyEvent};

use crate::messages::Messages;

use super::{Context, State};

impl super::Parser {
    pub fn parse_none_message(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        self.servers
            .get3()
            .update(&mut self.client, &mut self.user_dict);
        self.servers
            .get3()
            .draw(&self.grid, &mut self.out, &mut self.user_dict, &mut self.client);
        match self.servers.get3() {
            Messages::Unloaded(_) => {
                panic!("this should never happen!");
            }
            Messages::Loaded(_) => {}
            _ => {
                self.grid.context = Context::Channel;
                self.reset_all();
                return;
            }
        }
        match code {
            KeyCode::Left => {
                self.grid.context = Context::Channel;
                self.servers.get3().assume_loaded().flag();
                self.servers.get2().flag();
            }
            KeyCode::Up => {
                self.servers.get3().assume_loaded().up(&self.grid);
            }
            KeyCode::Down => {
                self.servers.get3().assume_loaded().down(&self.grid);
            }
            KeyCode::Enter => {
                self.servers.get3().assume_loaded().select();
                self.reset_all();
            }
            KeyCode::Char('t') => {
                self.state = State::Message;
            }
            KeyCode::Char('b') => {
                self.servers.get3().assume_loaded().back();
            }
            KeyCode::Char('c') => {
                self.servers
                    .get3()
                    .assume_loaded()
                    .color(&mut self.user_dict);
            }
            KeyCode::Char('m') => {
                self.message_person();
            }
            KeyCode::Char('o') => {
                self.servers
                    .get3()
                    .assume_loaded()
                    .open(&self.file_options, &self.grid);
            }
            _ => {}
        }
    }
}
