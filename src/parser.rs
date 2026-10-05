// parser.rs

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

pub fn parse_command(args: &[String]) -> Result<ParsedCommand, ParseError> {
    let mut parsed_args: Vec<String> = Vec::new();
    let mut expected_redirect = ExpectedRedirect::NoRedirect;
    let mut std_out: Option<Redirect> = None;
    let mut std_err: Option<Redirect> = None;
    if args.is_empty() {
        return Err(ParseError::NoCmd);
    }
    let cmd = args[0].clone();
    for arg in args[1..].iter() {
        match expected_redirect {
            ExpectedRedirect::Stdout => {
                std_out = Some(Redirect::Truncate(arg.clone()));
                expected_redirect = ExpectedRedirect::NoRedirect;
            }
            ExpectedRedirect::Stderr => {
                std_err = Some(Redirect::Truncate(arg.clone()));
                expected_redirect = ExpectedRedirect::NoRedirect;
            }
            ExpectedRedirect::StdoutAppend => {
                std_out = Some(Redirect::Append(arg.clone()));
                expected_redirect = ExpectedRedirect::NoRedirect;
            }
            ExpectedRedirect::StderrAppend => {
                std_err = Some(Redirect::Append(arg.clone()));
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
                    parsed_args.push(arg.clone());
                }
            },
        }
    }
    let result = match expected_redirect {
        ExpectedRedirect::NoRedirect => Ok(ParsedCommand {
            cmd: cmd,
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
    };
    result
}
