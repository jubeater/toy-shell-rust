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
    /// Carries the operator spelling (e.g. `"> or 1>"`) so the message names the
    /// exact form the user typed rather than a normalized one.
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

enum ExpectedRedirect {
    Stdout,
    StdoutAppend,
    Stderr,
    StderrAppend,
    NoRedirect,
}

pub fn parse_command(args: Vec<String>) -> Result<ParsedCommand, ParseError> {
    let mut parsed_args: Vec<String> = Vec::new();
    let mut expected_redirect = ExpectedRedirect::NoRedirect;
    let mut std_out: Option<Redirect> = None;
    let mut std_err: Option<Redirect> = None;

    let mut iter = args.into_iter();
    let cmd = iter.next().ok_or(ParseError::NoCmd)?;
    while let Some(arg) = iter.next() {
        match expected_redirect {
            ExpectedRedirect::Stdout => {
                std_out = Some(Redirect::Truncate(arg));
                expected_redirect = ExpectedRedirect::NoRedirect;
            }
            ExpectedRedirect::Stderr => {
                std_err = Some(Redirect::Truncate(arg));
                expected_redirect = ExpectedRedirect::NoRedirect;
            }
            ExpectedRedirect::StdoutAppend => {
                std_out = Some(Redirect::Append(arg));
                expected_redirect = ExpectedRedirect::NoRedirect;
            }
            ExpectedRedirect::StderrAppend => {
                std_err = Some(Redirect::Append(arg));
                expected_redirect = ExpectedRedirect::NoRedirect;
            }
            ExpectedRedirect::NoRedirect => match arg.as_str() {
                ">" | "1>" => {
                    expected_redirect = ExpectedRedirect::Stdout;
                }
                "2>" => {
                    expected_redirect = ExpectedRedirect::Stderr;
                }
                ">>" | "1>>" => {
                    expected_redirect = ExpectedRedirect::StdoutAppend;
                }
                "2>>" => {
                    expected_redirect = ExpectedRedirect::StderrAppend;
                }
                _ => {
                    parsed_args.push(arg);
                }
            },
        }
    }
    match expected_redirect {
        ExpectedRedirect::NoRedirect => Ok(ParsedCommand {
            cmd,
            args: parsed_args,
            stdout: std_out,
            stderr: std_err,
        }),
        ExpectedRedirect::Stdout => Err(ParseError::MissingFileName("> or 1>")),
        ExpectedRedirect::Stderr => Err(ParseError::MissingFileName("2>")),
        ExpectedRedirect::StdoutAppend => Err(ParseError::MissingFileName(">> or 1>>")),
        ExpectedRedirect::StderrAppend => Err(ParseError::MissingFileName("2>>")),
    }
}
