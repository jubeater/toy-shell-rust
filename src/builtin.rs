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

    pub fn execute(&self, args: &str) -> BuiltinResult {
        match self {
            Self::Exit => BuiltinResult::Exit,

            Self::Echo => {
                println!("{}", args.trim());
                BuiltinResult::Continue
            }

            Self::Pwd => {
                let path = env::current_dir().unwrap();
                println!("{}", path.to_string_lossy());
                BuiltinResult::Continue
            }

            Self::Cd => {
                if args.starts_with("~") {
                    if let Some(mut home) = env::home_dir() {
                        if args.len() > 1 {
                            home.push(&args[1..]);
                        };
                        env::set_current_dir(&home).unwrap();
                    } else {
                        eprintln!("Unable to detect the home directory.");
                    }
                } else if env::set_current_dir(args).is_err() {
                    println!("cd: {}: No such file or directory", args);
                }
                BuiltinResult::Continue
            }

            Self::Type => {
                if Self::from_cmd(&args).is_some() {
                    println!("{} is a shell builtin", args);
                } else if let Some(cmd_path) = find_executable(args) {
                    println!("{} is {}", args, cmd_path.to_string_lossy().into_owned());
                } else {
                    println!("{}: not found", args);
                }
                BuiltinResult::Continue
            }
        }
    }
}
