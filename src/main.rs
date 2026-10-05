use shell_words;
#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};

mod builtin;
mod executable;
mod parser;

use parser::{ParseError, ParsedCommand, Redirect};

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

enum ExpectedRedirect {
    Stdout,
    StdoutAppend,
    Stderr,
    StderrAppend,
    NoRedirect,
}

fn parse_command(args: &[String]) -> Result<parser::ParsedCommand, ParseError> {
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

// fn parse_command(input: &str) -> Vec<String> {
//     let mut args = Vec::new();
//     let mut current = String::new();
//     let mut in_single_quote: bool = false;
//     let mut in_double_quote: bool = false;
//     let mut in_escaping: bool = false;

//     for ch in input.chars() {
//         if in_single_quote || in_double_quote {
//             match ch {
//                 '\'' if !in_double_quote => {
//                     in_single_quote = false;
//                 }
//                 '\"' if !in_single_quote => {
//                     in_double_quote = false;
//                 }
//                 _ => {
//                     current.push(ch);
//                 }
//             }
//         } else if in_escaping {
//             current.push(ch);
//             in_escaping = false;
//         } else {
//             match ch {
//                 '\'' => {
//                     in_single_quote = true;
//                 }
//                 '\"' => {
//                     in_double_quote = true;
//                 }
//                 '\\' => {
//                     in_escaping = true;
//                 }
//                 ch if ch.is_whitespace() => {
//                     // finish current argument if appropriate
//                     let token = current.trim();
//                     if !token.is_empty() {
//                         args.push(token.to_string());
//                     }
//                     current = String::new();
//                 }

//                 _ => {
//                     current.push(ch);
//                 }
//             }
//         }
//     }

//     let token = current.trim();
//     if !token.is_empty() {
//         args.push(token.to_string());
//     }
//     args
// }
