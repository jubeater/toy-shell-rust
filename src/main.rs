use shell_words;
#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};

mod builtin;
mod executable;
mod parser;

use parser::{Redirect, parse_command};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let parts = shell_words::split(&input).unwrap();
        if let Ok(parsed) = parse_command(&parts) {
            match builtin::Builtin::from_cmd(&parsed.cmd) {
                Some(builtin) => {
                    let mut stdout: Box<dyn Write> = match &parsed.stdout {
                        None => Box::new(io::stdout()),
                        Some(Redirect::Truncate(path)) => {
                            Box::new(std::fs::File::create(path).unwrap())
                        }
                        Some(Redirect::Append(path)) => Box::new(
                            std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(path)
                                .unwrap(),
                        ),
                    };

                    let mut stderr: Box<dyn Write> = match &parsed.stderr {
                        None => Box::new(io::stderr()),
                        Some(Redirect::Truncate(path)) => {
                            Box::new(std::fs::File::create(path).unwrap())
                        }
                        Some(Redirect::Append(path)) => Box::new(
                            std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(path)
                                .unwrap(),
                        ),
                    };

                    match builtin.execute(&parsed.args, &mut stdout, &mut stderr) {
                        builtin::BuiltinResult::Continue => {}
                        builtin::BuiltinResult::Exit => break,
                    }
                }
                None => executable::execute_external(
                    &parsed.cmd,
                    &parsed.args,
                    parsed.stdout.as_ref(),
                    parsed.stderr.as_ref(),
                ),
            }
        }
    }
}
