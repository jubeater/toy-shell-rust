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

fn longest_common_prefix(matches: &[String]) -> String {
    let mut prefix = matches[0].clone();

    for s in &matches[1..] {
        while !s.starts_with(&prefix) {
            if prefix.is_empty() {
                return String::new();
            }
            prefix.pop(); // Remove the last character and retry
        }
    }

    prefix
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
    let mut previous_was_tab = false;
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
                    let mut matches: Vec<String> = builtin::Builtin::complete(&input)
                        .into_iter()
                        .map(str::to_owned)
                        .collect();

                    matches.extend(executable::complete(&input));

                    matches.sort();
                    matches.dedup();

                    match matches.as_slice() {
                        [] => {
                            print!("\x07");
                            previous_was_tab = false;
                        }

                        [completed] => {
                            if let Some(after) = completed.strip_prefix(&input) {
                                input.push_str(after);
                                input.push(' ');
                                print!("{after} ");
                            }

                            previous_was_tab = false;
                        }
                        _ => {
                            let lcp = longest_common_prefix(&matches);
                            if lcp.len() > input.len() {
                                // extend input to LCP
                                if let Some(after) = lcp.strip_prefix(&input) {
                                    input.push_str(after);
                                    print!("{after}");
                                }
                                previous_was_tab = false;
                            } else {
                                if previous_was_tab {
                                    print!("\r\n{}\r\n$ {}", matches.join("  "), input);
                                    previous_was_tab = false;
                                } else {
                                    print!("\x07");
                                    previous_was_tab = true;
                                }
                            }
                        }
                    }

                    io::stdout().flush()?;
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
