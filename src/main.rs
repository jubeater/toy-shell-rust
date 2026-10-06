use shell_words;
#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};

mod builtin;
mod executable;
mod parser;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use parser::{Redirect, parse_command};

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn main() {
    enable_raw_mode().unwrap();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        loop {
            // io::stdout().flush().unwrap();
            // let mut input = String::new();
            // io::stdin().read_line(&mut input).unwrap();
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char(c) => {
                        input.push(c);
                        print!("{c}");
                    }

                    KeyCode::Backspace => {
                        input.pop();
                    }

                    KeyCode::Tab => {
                        if let Some(completed) = builtin::Builtin::complete(&input) {
                            if let Some((_before, after)) = completed.split_once(&input) {
                                input.push_str(after);
                                print!("{after}");
                            }
                        }
                    }

                    KeyCode::Enter => {
                        break;
                    }

                    _ => {}
                }
            }
        }
        let parts = match shell_words::split(&input) {
            Ok(parts) => parts,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };
        let parsed = match parse_command(&parts) {
            Ok(parsed) => parsed,
            Err(_) => {
                eprintln!("err!");
                continue;
            }
        };
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

                match builtin.execute(&parsed.args, stdout.as_mut(), stderr.as_mut()) {
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
    disable_raw_mode().unwrap();
    let _raw_mode = RawModeGuard;
}
