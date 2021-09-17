use crossterm::event::{KeyCode, KeyEvent};

use crate::messages::Messages;

use super::{Context, State};

impl super::Parser {
    pub fn parse_none_message(&mut self, input: KeyEvent) {
        let KeyEvent { code, modifiers: _ } = input;
        self.servers
            .get3()
            .update(&self.io.tasks);
        self.servers
            .get3()
            .draw(&self.int.grid, &mut self.io.out, &mut self.int.user_dict, &self.io.tasks);
            match self.servers.get3() {
                Messages::Unloaded(_) => {
                    panic!("this should never happen!");
                }
                Messages::Loaded(_) => {}
                _ => {
                    self.int.grid.context = Context::Channel;
                    self.reset_all();
                    return;
                }
            }
        match code {
            KeyCode::Left => {
                self.int.grid.context = Context::Channel;
                self.servers.get3().assume_loaded().flag();
                self.servers.get2().flag();
            }
            KeyCode::Up => {
                self.servers.get3().assume_loaded().up(&self.int.grid);
            }
            KeyCode::Down => {
                self.servers.get3().assume_loaded().down(&self.int.grid);
            }
            KeyCode::Enter => {
                self.servers.get3().assume_loaded().select();
                self.reset_all();
            }
            KeyCode::Char('t') => {
                self.int.state = State::Message;
            }
            KeyCode::Char('b') => {
                self.servers.get3().assume_loaded().back();
            }
            KeyCode::Char('v') => {
                self.int.state = State::Visual;
            }
            KeyCode::Char('m') => {
                self.message_person();
            }
            KeyCode::Char('o') => {
                self.servers
                    .get3()
                    .assume_loaded()
                    .open(&self.int.file_options, &self.int.grid);
            }
            _ => {}
        }
    }
    pub fn parse_visual_messages(&mut self, input: KeyEvent) {
        let KeyEvent {code, modifiers: _} = input;
        self.servers
            .get3()
            .update(&self.io.tasks);
        self.servers
            .get3()
            .draw(&self.int.grid, &mut self.io.out, &mut self.int.user_dict, &self.io.tasks);
            match self.servers.get3() {
                Messages::Unloaded(_) => {
                    panic!("this should never happen!");
                }
                Messages::Loaded(_) => {}
                _ => {
                    self.int.grid.context = Context::Channel;
                    self.reset_all();
                    return;
                }
            }
        match code {
            KeyCode::Backspace | KeyCode::Esc | KeyCode::Enter => self.int.state = State::None,
            KeyCode::Left => {
                self.int.grid.context = Context::Channel;
                self.servers.get3().assume_loaded().flag();
                self.servers.get2().flag();
            }
            KeyCode::Up => {
                self.servers.get3().assume_loaded().up(&self.int.grid);
            }
            KeyCode::Down => {
                self.servers.get3().assume_loaded().down(&self.int.grid);
            }
            KeyCode::Char('r') => self.servers.get3().assume_loaded().red(&mut self.int.user_dict),
            KeyCode::Char('g') => self.servers.get3().assume_loaded().green(&mut self.int.user_dict),
            KeyCode::Char('b') => self.servers.get3().assume_loaded().blue(&mut self.int.user_dict),
            _ => {}
        }
    }
}
