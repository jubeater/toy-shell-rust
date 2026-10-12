use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;

/// A parsed command line, ready to dispatch.
#[derive(Debug)]
pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub stdout: Option<Redirect>,
    pub stderr: Option<Redirect>,
}

#[derive(Debug)]
pub enum Redirect {
    Truncate(String),
    Append(String),
}

impl Redirect {
    pub fn open(&self) -> io::Result<File> {
        match self {
            Self::Truncate(path) => File::create(path),
            Self::Append(path) => OpenOptions::new().create(true).append(true).open(path),
        }
    }
}

/// A command line that could not be turned into a runnable command.
#[derive(Debug)]
pub enum ParseError {
    /// The line was non-empty but contained no command word.
    NoCmd,
    /// A redirect operator was given without a file name.
    ///
    /// Carries an operator label (e.g. `"> or 1>"`) for the error message.
    MissingFileName(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCmd => write!(f, "no command given"),
            Self::MissingFileName(operator) => write!(f, "Missing file name for {operator}"),
        }
    }
}

impl std::error::Error for ParseError {}

enum Stream {
    Stdout,
    Stderr,
}

struct PendingRedirect {
    stream: Stream,
    append: bool,
    operator: &'static str,
}

impl PendingRedirect {
    fn into_redirect(self, file: String) -> (Stream, Redirect) {
        let redirect = if self.append {
            Redirect::Append(file)
        } else {
            Redirect::Truncate(file)
        };
        (self.stream, redirect)
    }
}

pub fn parse_command(args: Vec<String>) -> Result<ParsedCommand, ParseError> {
    let mut parsed_args: Vec<String> = Vec::new();
    let mut pending_redirect: Option<PendingRedirect> = None;
    let mut stdout: Option<Redirect> = None;
    let mut stderr: Option<Redirect> = None;

    let mut iter = args.into_iter();
    let cmd = iter.next().ok_or(ParseError::NoCmd)?;
    for arg in iter {
        // The token following a redirect operator is its filename.
        if let Some(pending_redirect) = pending_redirect.take() {
            let (stream, redirect) = pending_redirect.into_redirect(arg);
            match stream {
                Stream::Stdout => stdout = Some(redirect),
                Stream::Stderr => stderr = Some(redirect),
            }
            continue;
        }
        let (stream, append, operator) = match arg.as_str() {
            ">" | "1>" => (Stream::Stdout, false, "> or 1>"),
            "2>" => (Stream::Stderr, false, "2>"),
            ">>" | "1>>" => (Stream::Stdout, true, ">> or 1>>"),
            "2>>" => (Stream::Stderr, true, "2>>"),
            _ => {
                parsed_args.push(arg);
                continue;
            }
        };
        pending_redirect = Some(PendingRedirect {
            stream,
            append,
            operator,
        });
    }
    match pending_redirect {
        None => Ok(ParsedCommand {
            cmd,
            args: parsed_args,
            stdout,
            stderr,
        }),
        Some(pending_redirect) => Err(ParseError::MissingFileName(pending_redirect.operator)),
    }
}
