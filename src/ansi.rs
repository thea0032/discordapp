//Ansi stuff

pub const RESET: &str = "\u{001B}[0m\u{001B}[37m\u{001B}[40m";

pub const HIGH_INTENSITY: &str = "\u{001B}[1m";
pub const LOW_INTENSITY: &str = "\u{001B}[2m";

pub const ITALIC: &str = "\u{001B}[3m";
pub const UNDERLINE: &str = "\u{001B}[4m";
pub const BLINK: &str = "\u{001B}[5m";
pub const RAPID_BLINK: &str = "\u{001B}[6m";
pub const REVERSE_VIDEO: &str = "\u{001B}[7m";
pub const INVISIBLE_TEXT: &str = "\u{001B}[8m";

pub const BLACK: &str = "\u{001B}[30m";
pub const RED: &str = "\u{001B}[31m";
pub const GREEN: &str = "\u{001B}[32m";
pub const YELLOW: &str = "\u{001B}[33m";
pub const BLUE: &str = "\u{001B}[34m";
pub const MAGENTA: &str = "\u{001B}[35m";
pub const CYAN: &str = "\u{001B}[36m";
pub const WHITE: &str = "\u{001B}[37m";
pub const COLORS: &[&str] = &[WHITE, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN];

pub const BACKGROUND_BLACK: &str = "\u{001B}[40m";
pub const BACKGROUND_GREY: &str = "\u{001b}[48;5;237m";
pub const BACKGROUND_LIGHT_GREY: &str = "\u{001b}[48;5;240m";
pub const BACKGROUND_RED: &str = "\u{001B}[41m";
pub const BACKGROUND_GREEN: &str = "\u{001B}[42m";
pub const BACKGROUND_YELLOW: &str = "\u{001B}[43m";
pub const BACKGROUND_BLUE: &str = "\u{001B}[44m";
pub const BACKGROUND_MAGENTA: &str = "\u{001B}[45m";
pub const BACKGROUND_CYAN: &str = "\u{001B}[46m";
pub const BACKGROUND_WHITE: &str = "\u{001B}[47m";

// define your own strings and/or styles below
