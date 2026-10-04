#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};

mod builtin;
mod executable;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let whole_cmd = input.trim();
        let (cmd, args) = whole_cmd.split_once(' ').unwrap_or((whole_cmd, ""));
        match builtin::Builtin::from_cmd(cmd) {
            Some(builtin) => match builtin.execute(args) {
                builtin::BuiltinResult::Continue => {}
                builtin::BuiltinResult::Exit => break,
            },
            None => executable::execute_external(cmd, args),
        }
    }
}
