#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "exit" => {
                break;
            }
            e if e.starts_with("echo") => {
                println!("{}", &e[5..]);
            }
            e if e.starts_with("type") => {
                const BUILD_INS: [&str; 3] = ["echo", "exit", "type"];
                if BUILD_INS.contains(&&e[5..]) {
                    println!("{} is a shell builtin", &e[5..]);
                } else {
                    println!("{}: not found", &e[5..]);
                }
            }
            cmd => {
                println!("{}: command not found", cmd);
            }
        }
    }
}
