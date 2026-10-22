use crate::{ShellControl, executable::find_executable};
use std::{
    collections::HashMap,
    env,
    io::{self, Write},
};

pub type CompletionRegistry = HashMap<String, String>;

#[derive(Clone, Copy)]
pub enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Complete,
}

impl Builtin {
    /// Builtins available for lookup and completion. Add new variants here too.
    pub const ALL: &[Self] = &[
        Self::Exit,
        Self::Echo,
        Self::Type,
        Self::Pwd,
        Self::Cd,
        Self::Complete,
    ];

    /// The name this builtin is invoked as.
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
        Self::ALL
            .iter()
            .copied()
            .find(|builtin| builtin.name() == cmd)
    }

    pub fn execute(
        &self,
        args: &[String],
        registry: &mut CompletionRegistry,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> io::Result<ShellControl> {
        match self {
            Self::Exit => Ok(ShellControl::Exit),

            Self::Echo => {
                writeln!(stdout, "{}", args.join(" "))?;
                Ok(ShellControl::Continue)
            }

            Self::Pwd => {
                let path = env::current_dir()?;
                writeln!(stdout, "{}", path.to_string_lossy())?;
                Ok(ShellControl::Continue)
            }

            Self::Cd => {
                let Some(path) = args.first() else {
                    return Ok(ShellControl::Continue);
                };

                let target = if path == "~" || path.starts_with("~/") {
                    let Some(mut home) = env::home_dir() else {
                        writeln!(stderr, "Unable to detect the home directory.")?;
                        return Ok(ShellControl::Continue);
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

                Ok(ShellControl::Continue)
            }

            Self::Type => {
                let Some(cmd) = args.first() else {
                    return Ok(ShellControl::Continue);
                };

                if Self::from_cmd(cmd).is_some() {
                    writeln!(stdout, "{} is a shell builtin", cmd)?;
                } else if let Some(cmd_path) = find_executable(cmd) {
                    writeln!(stdout, "{} is {}", cmd, cmd_path.to_string_lossy())?;
                } else {
                    writeln!(stdout, "{}: not found", cmd)?;
                }
                Ok(ShellControl::Continue)
            }

            Self::Complete => {
                let Some(flag) = args.first() else {
                    return Ok(ShellControl::Continue);
                };
                match flag.as_str() {
                    "-p" => {
                        if let Some(cmd) = args.get(1) {
                            if let Some(path) = registry.get(cmd) {
                                writeln!(stdout, "complete -C {}: {}", path, cmd)?;
                            } else {
                                writeln!(stdout, "complete: {}: no completion specification", cmd)?;
                            }
                        } else {
                            writeln!(stdout, "complete: no cmd to register")?;
                        }
                    }
                    "-C" => {
                        if let Some(path) = args.get(1) {
                            if let Some(cmd) = args.get(2) {
                                registry.insert(cmd.clone(), path.clone());
                            } else {
                                writeln!(stdout, "complete: no cmd found")?;
                            }
                        } else {
                            writeln!(stdout, "complete: no path found")?;
                        }
                    }
                    _ => {}
                }

                Ok(ShellControl::Continue)
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
