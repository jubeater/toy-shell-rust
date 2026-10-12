use crate::executable::find_executable;
use std::{env, io, io::Write};

#[derive(Clone, Copy)]
pub enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Complete,
}
pub enum BuiltinResult {
    Continue,
    Exit,
}

impl Builtin {
    pub const ALL: &[Self] = &[
        Self::Exit,
        Self::Echo,
        Self::Type,
        Self::Pwd,
        Self::Cd,
        Self::Complete,
    ];

    /// The name this builtin is invoked as.
    ///
    /// Exhaustive by construction: adding a variant fails to compile until it is
    /// named here, which keeps `ALL`, `from_cmd`, and `complete` in sync.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::Echo => "echo",
            Self::Type => "type",
            Self::Pwd => "pwd",
            Self::Cd => "cd",
            Self::Complete => "complete",
        }
    }

    pub fn from_cmd(cmd: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|builtin| builtin.name() == cmd)
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
                let Some(path) = args.first() else {
                    return Ok(BuiltinResult::Continue);
                };

                let target = if path == "~" || path.starts_with("~/") {
                    let Some(mut home) = env::home_dir() else {
                        writeln!(stderr, "Unable to detect the home directory.")?;
                        return Ok(BuiltinResult::Continue);
                    };

                    if let Some(relative) = path.strip_prefix("~/") {
                        home.push(relative);
                    }

                    home
                } else {
                    path.into()
                };

                if env::set_current_dir(&target).is_err() {
                    writeln!(stdout, "cd: {}: No such file or directory", path)?;
                }

                Ok(BuiltinResult::Continue)
            }

            Self::Type => {
                let Some(cmd) = args.first() else {
                    return Ok(BuiltinResult::Continue);
                };

                if Self::from_cmd(cmd).is_some() {
                    writeln!(stdout, "{} is a shell builtin", cmd)?;
                } else if let Some(cmd_path) = find_executable(cmd) {
                    writeln!(stdout, "{} is {}", cmd, cmd_path.to_string_lossy())?;
                } else {
                    writeln!(stdout, "{}: not found", cmd)?;
                }
                Ok(BuiltinResult::Continue)
            }

            Self::Complete => {
                let Some(flag) = args.first() else {
                    return Ok(BuiltinResult::Continue);
                };
                match flag.as_str() {
                    "-p" => {
                        if let Some(cmd) = args.get(1) {
                            writeln!(stdout, "complete: {}: no completion specification", cmd)?;
                        }
                    }
                    "-C" => {
                        todo!()
                    }
                    _ => {}
                }

                Ok(BuiltinResult::Continue)
            }
        }
    }

    pub fn complete(prefix: &str) -> Vec<&'static str> {
        Self::ALL
            .iter()
            .map(|builtin| builtin.name())
            .filter(|name| name.starts_with(prefix))
            .collect()
    }
}
