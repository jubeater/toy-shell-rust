#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};

mod builtin;
mod executable;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let whole_cmd = input.trim();

        let tokens = parse_command(whole_cmd);

        let Some((cmd, args)) = tokens.split_first() else {
            continue;
        };
        match builtin::Builtin::from_cmd(cmd) {
            Some(builtin) => match builtin.execute(args) {
                builtin::BuiltinResult::Continue => {}
                builtin::BuiltinResult::Exit => break,
            },
            None => executable::execute_external(cmd, args),
        }
    }
}

fn parse_command(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote: bool = false;
    let mut in_double_quote: bool = false;
    let mut in_escaping: bool = false;

    for ch in input.chars() {
        if in_single_quote || in_double_quote {
            match ch {
                '\'' if !in_double_quote => {
                    in_single_quote = false;
                }
                '\"' if !in_single_quote => {
                    in_double_quote = false;
                }
                _ => {
                    current.push(ch);
                }
            }
        } else if in_escaping {
            current.push(ch);
            in_escaping = false;
        } else {
            match ch {
                '\'' => {
                    in_single_quote = true;
                }
                '\"' => {
                    in_double_quote = true;
                }
                '\\' => {
                    in_escaping = true;
                }
                ch if ch.is_whitespace() => {
                    // finish current argument if appropriate
                    let token = current.trim();
                    if !token.is_empty() {
                        args.push(token.to_string());
                    }
                    current = String::new();
                }

                _ => {
                    current.push(ch);
                }
            }
        }
    }

    let token = current.trim();
    if !token.is_empty() {
        args.push(token.to_string());
    }
    args
}
