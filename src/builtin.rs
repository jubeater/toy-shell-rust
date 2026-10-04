use crate::executable::find_executable;
use std::env;
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

    pub fn execute(&self, args: &[String]) -> BuiltinResult {
        match self {
            Self::Exit => BuiltinResult::Exit,

            Self::Echo => {
                println!("{}", args.join(" "));
                BuiltinResult::Continue
            }

            Self::Pwd => {
                let path = env::current_dir().unwrap();
                println!("{}", path.to_string_lossy());
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
                            eprintln!("Unable to detect the home directory.");
                        }
                    } else if env::set_current_dir(path).is_err() {
                        println!("cd: {}: No such file or directory", path);
                    }
                }
                BuiltinResult::Continue
            }

            Self::Type => {
                if let Some(cmd) = args.first() {
                    if Self::from_cmd(cmd).is_some() {
                        println!("{} is a shell builtin", cmd);
                    } else if let Some(cmd_path) = find_executable(cmd) {
                        println!("{} is {}", cmd, cmd_path.to_string_lossy().into_owned());
                    } else {
                        println!("{}: not found", cmd);
                    }
                }
                BuiltinResult::Continue
            }
        }
    }
}
