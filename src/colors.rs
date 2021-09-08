
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Color {
    red: usize,
    blue: usize,
    green: usize,
}
impl Color {
    pub fn new() -> Color {
        Color { red: 5, blue: 5, green: 5 }
    }
    pub fn to_ansi_value(&self) -> String {
        return format!("\u{001b}[38;5;{}m", 16 + self.blue + self.green * 6 + self.red * 36);
    }
    pub fn red(&mut self) {
        self.red += 1;
        if self.red == 6 {
            self.red = 0;
        }
    }
    pub fn blue(&mut self) {
        self.blue += 1;
        if self.blue == 6 {
            self.blue = 0;
        }
    }
    pub fn green(&mut self) {
        self.green += 1;
        if self.green == 6 {
            self.green = 0;
        }
    }
}