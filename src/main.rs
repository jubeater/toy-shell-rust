use shell_words;
#[allow(unused_imports)]
use std::io::{self, IsTerminal, Write};

use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, Editor};

mod builtin;
mod completion;
mod executable;
mod parser;

use completion::ShellHelper;
use parser::{Redirect, parse_command};

type ShellEditor = Editor<ShellHelper, DefaultHistory>;

fn read_input(interactive: bool, editor: &mut ShellEditor) -> io::Result<Option<String>> {
    if interactive {
        return match editor.readline("$ ") {
            Ok(line) => Ok(Some(line)),
            Err(ReadlineError::Interrupted) => Ok(Some(String::new())),
            Err(ReadlineError::Eof) => Ok(None),
            Err(err) => Err(io::Error::other(err)),
        };
    }

    print!("$ ");
    io::stdout().flush()?;

    let mut input = String::new();

    match io::stdin().read_line(&mut input)? {
        0 => Ok(None),
        _ => Ok(Some(input)),
    }
}

fn main() -> io::Result<()> {
    let interactive = io::stdin().is_terminal();

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut editor = ShellEditor::with_config(config).map_err(io::Error::other)?;

    editor.set_helper(Some(ShellHelper::new()));
    loop {
        let Some(input) = read_input(interactive, &mut editor)? else {
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
