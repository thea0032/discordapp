use std::{io::Stdout, mem::replace};

use crossterm::{
    cursor::MoveTo,
    queue, style,
};

pub struct Textbox {
    text: Vec<Vec<char>>,
    cursor_line: usize,
    cursor_pos: usize,
    pub flag: bool,
    length: usize,
}
impl Textbox {
    pub fn new(length: usize) -> Textbox {
        Textbox {
            text: vec![Vec::new()],
            cursor_line: 0,
            cursor_pos: 0,
            flag: true,
            length,
        }
    }
    pub fn add_to_end(&mut self, lines: Vec<String>) {
        self.flag = true;
        for line in lines {
            self.text
                .last_mut()
                .expect("illegal state")
                .append(&mut line.chars().collect());
                self.text.push(Vec::new());
        }
        self.text.pop();
    }
    pub fn add_char(&mut self, c: char) {
        self.text[self.cursor_line].insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.flag = true;
    }
    pub fn backspace(&mut self) -> Option<char> {
        self.flag = true;
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            Some(self.text[self.cursor_line].remove(self.cursor_pos))
        } else if self.cursor_line > 0 {
            let mut line = self.text.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_pos = self.text[self.cursor_line].len();
            self.text[self.cursor_line].append(&mut line);
            Some('\n')
        } else {
            None
        }
    }
    pub fn delete(&mut self) -> Option<char> {
        self.flag = true;
        if self.cursor_pos < self.text[self.cursor_line].len() {
            Some(self.text[self.cursor_line].remove(self.cursor_pos))
        } else if self.cursor_line < self.text.len() - 1 {
            let mut line = self.text.remove(self.cursor_line + 1);
            self.text[self.cursor_line].append(&mut line);
            Some('\n')
        } else {
            None
        }
    }
    pub fn draw(&mut self, start_x: u16, start_y: u16, out: &mut Stdout, force_cursor_move: bool) -> bool {
        if self.flag {
            self.draw_real(start_x, start_y, out);
            self.flag = false;
            true
        } else if force_cursor_move {
            let _ = queue!(
                out,
                MoveTo(
                    start_x + (self.cursor_pos % self.length) as u16,
                    start_y + (self.cursor_line + self.cursor_pos / self.length) as u16
                )
            );
            true
        } else {
            false
        }
    }
    fn draw_real(&mut self, start_x: u16, start_y: u16, out: &mut Stdout) {
        let mut i = 0;
        for mut line in &self.text {
            let placeholder = vec![' '];
            if line.is_empty() {
                line = &placeholder;
            }
            for block in line.chunks(self.length) {
                let temp = " ".to_string();
                let space_stock = temp.chars().cycle();
                let printval = block
                    .iter()
                    .copied()
                    .chain(space_stock)
                    .take(self.length)
                    .collect::<String>();
                let _ = queue!(out, MoveTo(start_x, i + start_y));
                let _ = queue!(out, style::Print(printval));
                i += 1;
            }
        }
        let _ = queue!(
            out,
            MoveTo(
                start_x + (self.cursor_pos as u16),
                start_y + (self.cursor_line as u16)
            )
        );
    }
    pub fn lines(&mut self) -> usize {
        let mut i = 0;
        for line in &self.text {
            if line.is_empty() {
                i += 1;
                continue;
            }
            i += (line.len() - 1) / self.length;
        }
        i
    }
    pub fn flush(&mut self) -> String {
        self.flag = true;
        self.cursor_pos = 0;
        self.cursor_line = 0;
        replace(&mut self.text, vec![Vec::new()])
            .join(&'\n')
            .into_iter()
            .collect()
    }
    pub fn newline(&mut self) {
        self.flag = true;
        let newvec: Vec<char> = self.text[self.cursor_line]
            .splice(self.cursor_pos.., Vec::new())
            .collect();
        self.cursor_line += 1;
        self.cursor_pos = 0;
        self.text.insert(self.cursor_line, newvec);
    }
    pub fn up(&mut self) {
        self.flag = true;
        if self.cursor_pos >= self.length {
            self.cursor_pos -= self.length;
        } else if self.cursor_line == 0 {
            self.cursor_pos = 0;
        } else {
            self.cursor_line -= 1;
            self.cursor_pos %= self.length;
            if self.text[self.cursor_line].len() > self.length {
                while self.cursor_pos < self.text[self.cursor_line].len() - self.length {
                    self.cursor_pos += self.length;
                }
            }
            if self.cursor_pos > self.text[self.cursor_line].len() {
                self.cursor_pos = self.text[self.cursor_line].len();
            }
        }
    }
    pub fn down(&mut self) {
        self.flag = true;
        if self.text[self.cursor_line].len() - self.cursor_pos >= self.length {
            self.cursor_pos += self.length;
        } else if self.cursor_line == self.text.len() - 1 {
            self.cursor_pos = self.text[self.cursor_line].len();
        } else {
            self.cursor_line += 1;
            self.cursor_pos %= self.length;
            if self.cursor_pos > self.text[self.cursor_line].len() {
                self.cursor_pos = self.text[self.cursor_line].len();
            }
        }
    }
    pub fn left(&mut self) {
        self.flag = true;
        if self.cursor_pos == 0 && self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_pos = self.text[self.cursor_line].len();
        } else if self.cursor_pos != 0 {
            self.cursor_pos -= 1;
        }
    }
    pub fn right(&mut self) {
        self.flag = true;
        if self.cursor_pos == self.text[self.cursor_line].len()
            && self.cursor_line < self.text.len() - 1
        {
            self.cursor_line += 1;
            self.cursor_pos = 0;
        } else if self.cursor_pos < self.text[self.cursor_line].len() {
            self.cursor_pos += 1;
        }
    }
    pub fn flag(&mut self) {
        self.flag = true;
    }
}
