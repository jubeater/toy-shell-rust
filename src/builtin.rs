use crate::executable::find_executable;
use std::{env, io::Write};
pub enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}
pub enum BuiltinResult {
    Continue,
    Exit,
}

impl Builtin {
    pub const ALL: &'static [&'static str] = &["exit", "echo", "type", "pwd", "cd"];
    pub fn from_cmd(cmd: &str) -> Option<Self> {
        match cmd {
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "type" => Some(Self::Type),
            "pwd" => Some(Self::Pwd),
            "cd" => Some(Self::Cd),
            _ => None,
        }
    }

    pub fn _name(&self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::Echo => "echo",
            Self::Type => "type",
            Self::Pwd => "pwd",
            Self::Cd => "cd",
        }
    }

    pub fn execute(
        &self,
        args: &[String],
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> BuiltinResult {
        match self {
            Self::Exit => BuiltinResult::Exit,

            Self::Echo => {
                writeln!(stdout, "{}", args.join(" ")).unwrap();
                BuiltinResult::Continue
            }

            Self::Pwd => {
                let path = env::current_dir().unwrap();
                writeln!(stdout, "{}", path.to_string_lossy()).unwrap();
                BuiltinResult::Continue
            }

            Self::Cd => {
                if let Some(path) = args.first() {
                    if path.starts_with('~') {
                        if let Some(mut home) = env::home_dir() {
                            if args.len() > 1 {
                                home.push(&path[1..]);
                            };
                            env::set_current_dir(&home).unwrap();
                        } else {
                            writeln!(stderr, "Unable to detect the home directory.").unwrap();
                        }
                    } else if env::set_current_dir(path).is_err() {
                        writeln!(stdout, "cd: {}: No such file or directory", path).unwrap();
                    }
                }
                BuiltinResult::Continue
            }

            Self::Type => {
                if let Some(cmd) = args.first() {
                    if Self::from_cmd(cmd).is_some() {
                        writeln!(stdout, "{} is a shell builtin", cmd).unwrap();
                    } else if let Some(cmd_path) = find_executable(cmd) {
                        writeln!(
                            stdout,
                            "{} is {}",
                            cmd,
                            cmd_path.to_string_lossy().into_owned()
                        )
                        .unwrap();
                    } else {
                        writeln!(stdout, "{}: not found", cmd).unwrap();
                    }
                }
                BuiltinResult::Continue
            }
        }
    }

    pub fn complete(prefix: &str) -> Vec<&'static str> {
        Self::ALL
            .iter()
            .copied()
            .filter(|name| name.starts_with(prefix))
            .collect()
    }
}
