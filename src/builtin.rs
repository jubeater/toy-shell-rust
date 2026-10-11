// builtin.rs
use crate::executable::find_executable;
use std::{env, io, io::Write};
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
    ) -> io::Result<BuiltinResult> {
        match self {
            Self::Exit => Ok(BuiltinResult::Exit),

            Self::Echo => {
                writeln!(stdout, "{}", args.join(" "))?;
                Ok(BuiltinResult::Continue)
            }

            Self::Pwd => {
                let path = env::current_dir()?;
                writeln!(stdout, "{}", path.to_string_lossy())?;
                Ok(BuiltinResult::Continue)
            }

            Self::Cd => {
                if let Some(path) = args.first() {
                    if path.starts_with('~') {
                        if let Some(mut home) = env::home_dir() {
                            if args.len() > 1 {
                                home.push(&path[1..]);
                            };
                            env::set_current_dir(&home)?;
                        } else {
                            writeln!(stderr, "Unable to detect the home directory.")?;
                        }
                    } else if env::set_current_dir(path).is_err() {
                        writeln!(stdout, "cd: {}: No such file or directory", path)?;
                    }
                }
                Ok(BuiltinResult::Continue)
            }

            Self::Type => {
                if let Some(cmd) = args.first() {
                    if Self::from_cmd(cmd).is_some() {
                        writeln!(stdout, "{} is a shell builtin", cmd)?;
                    } else if let Some(cmd_path) = find_executable(cmd) {
                        writeln!(
                            stdout,
                            "{} is {}",
                            cmd,
                            cmd_path.to_string_lossy().into_owned()
                        )?;
                    } else {
                        writeln!(stdout, "{}: not found", cmd)?;
                    }
                }
                Ok(BuiltinResult::Continue)
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
