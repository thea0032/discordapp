use std::fs::{self, write};
use std::io::{self, Write, stdin, stdout};
use std::process::Command;

use crate::ansi;

#[cfg(target_os = "windows")]
pub const SEP:char = '\\';
#[cfg(not(target_os = "windows"))]
pub const SEP:char = '/';

pub fn get_str() -> String {
    let mut buffer = String::new();
    println!("Couldn't find token in filesystem! You will now be prompted to enter the token manually.");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut buffer);
    if let Some('\n') = buffer.chars().last() {
        buffer.pop();
    }
    if let Some('\r') = buffer.chars().last() {
        buffer.pop();
    }
    buffer
}
pub fn fs_write<> (url: &str) -> Option<(String, bool)> {
    let mut p =  std::env::current_dir().ok()?;
    p.push("output");
    p.push(url);
    let p_clone = p.clone();
    let s = p_clone.as_os_str();
    match std::fs::read(s) {
        Ok(_) => s.to_str().and_then(|x| (Some((x.to_string(), false)))),
        Err(_) => {
           s.to_str().and_then(|x| Some((x.to_string(), true)))
        },
    }
}
pub fn fs_write_2(value: Vec<u8>, url: &str) {
    let mut p =  std::env::current_dir().ok().expect("Current directory cannot be found!");
    p.push("output");
    p.push(url);
    write(p, value).expect("could not write to file!");
}
///debug function. Don't touch unless you know what you're doing. It's not being debugged; it's for debugging. 
pub fn add_on(file: &str, value: &str) {
    std::fs::write(file, std::fs::read_to_string(file).unwrap_or(String::new()) + "\n" + value).expect("Nobody cares, you moron");
}
pub fn open_with(file: &str, program: &str) {
    Command::new(program).arg(file).spawn().expect("FAILURE");
}
pub fn run(program: &str) {
    Command::new(program).spawn().expect("FAILURE");
}
pub fn current_dir() -> String {
    std::env::current_dir().expect("FAILED").to_str().unwrap().to_string()
}
pub fn from_relative_path(string: String) -> String {
    let mut temp = std::env::current_dir().expect("FAILED");
    temp.push(string);
    temp.to_str().unwrap().to_string()
}
pub fn get_file_root(prompt: &str) -> io::Result<String> {
    get_file("\\", prompt)
}
pub fn get_file(current: &str, prompt: &str) -> io::Result<String> {
    let mut path: String = current.to_string();
    loop {
        print!("{}", ansi::RESET);
        println!("{}", prompt);
        println!("Your current path: {}", path);
        println!("Select a file from the current directory by typing it. ");
        println!("Typing a directory (marked in blue) will enter it. ");
        println!("Typing \".\" will exit the current directory if possible. ");
        println!("Files: ");
        let mut folders: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();
        let temp = fs::read_dir(&path)?;
        for line in temp {
            let line = line?;
            let typ = line.file_type()?;
            if typ.is_dir() {
                print!("{}", ansi::BLUE);
                folders.push(line.file_name().to_str().unwrap().to_string());
            } else {
                print!("{}", ansi::GREEN);
                files.push(line.file_name().to_str().unwrap().to_string());
            }
            println!("{}", line.file_name().to_str().unwrap());
        } // prints out the current directory
        let input = get_str();
        if input == "." {
            let mut temp = path.split(SEP).collect::<Vec<&str>>();
            temp.pop();
            temp.pop();
            path = temp.join(&*vec![SEP].iter().collect::<String>());
            path.push(SEP);
        } else if folders.contains(&input) {
            path.push_str(&input);
            path.push(SEP);
        } else if files.contains(&input) {
            return Ok(path + &input);
        }
    }
}
pub struct FileOptions {
    pub browser: String,
}
impl FileOptions {
    pub fn new() -> FileOptions {
        //let browser = get_file_root("please choose the browser you would like to use with this interface.").expect("failed to select a browser");
        FileOptions {
            browser: "\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe".to_string(),
        }
    }
    pub fn open(&self, file: &str) {
        let extension = file.split('.').last(); // the extension will be used later. 
        Command::new(&self.browser).arg(file).spawn().expect("Could not open file!");
    }
}