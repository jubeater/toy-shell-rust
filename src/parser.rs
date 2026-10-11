use std::fs::{File, OpenOptions};
use std::io;

pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub stdout: Option<Redirect>,
    pub stderr: Option<Redirect>,
}

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

pub enum ParseError {
    NoCmd,
    MissingFileName(String),
}

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
        ExpectedRedirect::Stdout => Err(ParseError::MissingFileName(
            "Missing file name for > or 1>".to_string(),
        )),
        ExpectedRedirect::Stderr => Err(ParseError::MissingFileName(
            "Missing file name for 2>".to_string(),
        )),
        ExpectedRedirect::StdoutAppend => Err(ParseError::MissingFileName(
            "Missing file name for >> or 1>>".to_string(),
        )),
        ExpectedRedirect::StderrAppend => Err(ParseError::MissingFileName(
            "Missing file name for 2>>".to_string(),
        )),
    }
}
