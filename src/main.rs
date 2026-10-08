use shell_words;
#[allow(unused_imports)]
use std::env;
use std::io::{self, IsTerminal, Write};

mod builtin;
mod executable;
mod parser;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use parser::{Redirect, parse_command};

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn read_input(interactive: bool) -> io::Result<Option<String>> {
    let mut input = String::new();
    if !interactive {
        return match io::stdin().read_line(&mut input)? {
            0 => Ok(None),
            _ => Ok(Some(input)),
        };
    }

    enable_raw_mode()?;
    let _raw_mode = RawModeGuard;
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                // Raw-mode LF is decoded as Ctrl+J; CR is decoded as Enter.
                KeyCode::Char('j' | 'm') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    print!("\r\n");
                    io::stdout().flush()?;
                    return Ok(Some(input));
                }

                KeyCode::Char('d')
                    if key.modifiers.contains(KeyModifiers::CONTROL) && input.is_empty() =>
                {
                    return Ok(None);
                }

                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    print!("^C\r\n");
                    io::stdout().flush()?;
                    return Ok(Some(String::new()));
                }

                KeyCode::Char(c)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    input.push(c);
                    print!("{c}");
                    io::stdout().flush()?;
                }

                KeyCode::Backspace => {
                    // input.pop();
                }

                KeyCode::Tab => {
                    if let Some(completed) = builtin::Builtin::complete(&input) {
                        if let Some((_before, after)) = completed.split_once(&input) {
                            input.push_str(after);
                            input.push(' ');
                            print!("{after} ");
                            io::stdout().flush()?;
                        }
                    } else {
                        // print bell charactor to indicate no match for auto complete
                        print!("\x07");
                        io::stdout().flush()?;
                    }
                }

                KeyCode::Enter => {
                    print!("\r\n");
                    io::stdout().flush()?;
                    return Ok(Some(input));
                }

                _ => {}
            }
        }
    }
}

fn main() -> io::Result<()> {
    let interactive = io::stdin().is_terminal();
    loop {
        print!("$ ");
        io::stdout().flush()?;
        let Some(input) = read_input(interactive)? else {
            break;
        };
        if input.trim().is_empty() {
            continue;
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
    Ok(())
}
