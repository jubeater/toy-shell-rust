use std::io::{self, IsTerminal, Write};

use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, Editor};

mod builtin;
mod completion;
mod executable;
mod parser;
mod registry;

use completion::ShellHelper;
use parser::{Redirect, parse_command};
use registry::CompletionRegistry;

type ShellEditor = Editor<ShellHelper, DefaultHistory>;

/// Loop control for the REPL: `Continue` keeps prompting, `Exit` breaks out.
pub(crate) enum ShellControl {
    Continue,
    Exit,
}

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

fn open_stream(
    redirect: Option<&Redirect>,
    default: fn() -> Box<dyn Write>,
) -> io::Result<Box<dyn Write>> {
    match redirect {
        None => Ok(default()),
        Some(redirect) => Ok(Box::new(redirect.open()?)),
    }
}

fn run_line(input: &str, registry: &mut CompletionRegistry) -> io::Result<ShellControl> {
    if input.trim().is_empty() {
        return Ok(ShellControl::Continue);
    }
    let parts = match shell_words::split(input) {
        Ok(parts) => parts,
        Err(err) => {
            eprintln!("{err}");
            return Ok(ShellControl::Continue);
        }
    };
    let parsed = match parse_command(parts) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("{err}");
            return Ok(ShellControl::Continue);
        }
    };
    match builtin::Builtin::from_cmd(&parsed.cmd) {
        Some(builtin) => {
            let mut stdout = open_stream(parsed.stdout.as_ref(), || Box::new(io::stdout()))?;
            let mut stderr = open_stream(parsed.stderr.as_ref(), || Box::new(io::stderr()))?;

            builtin.execute(&parsed.args, registry, stdout.as_mut(), stderr.as_mut())
        }
        None => {
            executable::execute_external(
                &parsed.cmd,
                &parsed.args,
                parsed.stdout.as_ref(),
                parsed.stderr.as_ref(),
            )?;
            Ok(ShellControl::Continue)
        }
    }
}

fn main() -> io::Result<()> {
    let interactive = io::stdin().is_terminal();

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut editor = ShellEditor::with_config(config).map_err(io::Error::other)?;
    editor.set_helper(Some(ShellHelper::new()));
    let mut registry = CompletionRegistry::new();
    while let Some(input) = read_input(interactive, &mut editor)? {
        match run_line(&input, &mut registry)? {
            ShellControl::Continue => {}
            ShellControl::Exit => break,
        }
    }
    Ok(())
}
