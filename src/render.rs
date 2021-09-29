use std::io::Stdout;

use crate::input::Context;

pub struct Grid {
    pub start_y: usize,
    pub border_y: usize,
    pub end_y: usize,
    pub max_box_len: usize, // maximum length of text box
    pub start_x: usize,     // beginning of screen -> servers (usually 0)
    pub border_1: usize,    // servers -> categories
    pub border_2: usize,    // categories -> channels
    pub border_3: usize,    // channels -> messages
    pub end_x: usize,       // messages -> end of screen
    pub context: Context,
}
impl Grid {
    pub fn new(max_x: usize, max_y: usize) -> Grid {
        Grid {
            start_y: 0,
            border_y: (max_y - 1) as usize,
            end_y: max_y as usize,
            max_box_len: 15.min(max_y / 2).max(1) as usize,
            start_x: 0,
            border_1: 25.min((max_x / 5) as usize),
            border_2: 50.min((max_x * 2 / 5) as usize),
            border_3: 75.min((max_x * 3 / 5) as usize),
            end_x: max_x as usize,
            context: Context::Server,
        }
    }
    pub fn len_servers(&self) -> usize {
        self.border_1
    }
    pub fn len_categories(&self) -> usize {
        self.border_2 - self.border_1
    }
    pub fn len_channels(&self) -> usize {
        self.border_3 - self.border_2
    }
    pub fn len_messages(&self) -> usize {
        self.end_x - self.border_3
    }
    pub fn height(&self) -> usize {
        self.border_y - self.start_y
    }
    pub fn set_lines(&mut self, lines: usize) {
        self.border_y = self.end_y - lines;
    }
    pub fn servers_selected(&self) -> bool {
        matches!(self.context, Context::Server)
    }
    pub fn categories_selected(&self) -> bool {
        matches!(self.context, Context::Category)
    }
    pub fn channels_selected(&self) -> bool {
        matches!(self.context, Context::Channel)
    }
    pub fn messages_selected(&self) -> bool {
        matches!(self.context, Context::Message)
    }
    pub fn total_across(&self) -> usize {
        self.end_x - self.start_x
    }
    pub fn update(&mut self, lines: usize, max_y: usize, max_x: usize) {
        self.border_y = self.end_y - lines;
        self.end_y = max_y;
        self.max_box_len = 15.min(max_y / 2).max(1);
        self.border_1 = 25.min(max_x / 5);
        self.border_2 = 50.min(max_x * 2 / 5);
        self.border_3 = 75.min(max_x * 3 / 5);
        self.end_x = max_x;
    }
    pub fn update_msg(&mut self, lines: usize) {
        self.border_y = self.end_y - lines;
    }
}