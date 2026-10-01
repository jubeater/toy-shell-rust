#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let cmd = command.trim();
        if cmd == "exit" {
            break;
        }
        println!("{}: command not found", cmd);
    }
}
