use std::fs::{self, read, write};
use std::io::{self, stdin, stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::Receiver;

use serenity::model::event::Event;

use crate::ansi;
use crate::render::Grid;

#[cfg(target_os = "windows")]
pub const SEP: char = '\\';
#[cfg(not(target_os = "windows"))]
pub const SEP: char = '/';
pub fn get_str(msg: &str) -> String {
    let mut buffer = String::new();
    print!("{}", msg);
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
pub fn fs_write(url: &str) -> (String, bool) {
    let mut p = std::env::current_dir().expect("Could not find the current directory!");
    p.push("output");
    p.push(url);
    let p_clone = p.clone();
    let s = p_clone.as_os_str();
    (s.to_str().unwrap().to_string(), read(s).is_err())
}
pub fn fs_write_2(value: Vec<u8>, location: &str) {
    let mut p = std::env::current_dir().expect("Current directory cannot be found!");
    p.push("output");
    p.push(location);
    write(p, value).expect("could not write to file!");
}
///debug function. Don't touch unless you know what you're doing. It's not being debugged; it's for debugging.
pub fn add_on(file: &str, value: &str) {
    std::fs::write(
        file,
        std::fs::read_to_string(file).unwrap_or(String::new()) + "\n" + value,
    )
    .expect("Nobody cares, you moron");
}
pub fn open_with(file: &str, program: &str) {
    Command::new(program).arg(file).spawn().expect("FAILURE");
}
pub fn run(program: &str) {
    Command::new(program).spawn().expect("FAILURE");
}
pub fn current_dir() -> String {
    std::env::current_dir()
        .expect("FAILED")
        .to_str()
        .unwrap()
        .to_string()
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
        let input = get_str("");
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
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ExtConfig {
    // the path to the default application to open files
    pub default_path: String,
    // the file extensions w/ different programs
    pub extensions: Vec<Vec<String>>,
    // the corresponding paths to the programs associated with the file extensions
    pub paths: Vec<String>,
}
impl ExtConfig {
    pub fn new() -> ExtConfig {
        let mut path = PathBuf::new();
        path.push("save");
        path.push("config.json");
        if let Ok(val) = fs::read(&path) {
            if let Ok(val) = serde_json::from_slice(&val) {
                return val;
            }
        }
        let v = get_file(&current_dir(), "Could not read/parse config file! Please enter an application to open files with:").expect("Could not read file!");
        let f = ExtConfig {
            default_path: v,
            extensions: Vec::new(),
            paths: Vec::new(),
        };
        let _ = fs::write(&path, serde_json::to_string(&f).expect("Could not convert!"));
        f
    }
    pub fn open(&self, file: &str) {
        let extension = file.split('.').last(); // the extension will be used later.
        let mut v: Option<usize> = None; 
        if let Some(val) = extension {
            for (i, line) in self.extensions.iter().enumerate() {
                if line.iter().any(|x| x == val) {
                    v = Some(i); 
                    break;
                }
            }
        }
        Command::new(if let Some(val) = v {
            &self.paths[val]
        } else {
            &self.default_path
        })
            .arg(file)
            .spawn()
            .expect("Could not open file!");
    }
    pub fn edit(&mut self, recv:Receiver<Event>, grid: &mut Grid) {
        loop {
            
        }
    }
}
