#[allow(unused_imports)]
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let whole_cmd = input.trim();
        let (cmd, args) = whole_cmd.split_once(' ').unwrap_or((whole_cmd, ""));
        match cmd {
            "exit" => {
                break;
            }
            "echo" => {
                println!("{}", args.trim());
            }
            "type" => {
                const BUILDIN_CMDS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];
                if BUILDIN_CMDS.contains(&args) {
                    println!("{} is a shell builtin", args);
                } else if let Some(cmd_path) = is_valid_cmd(args) {
                    println!("{} is {}", args, cmd_path);
                } else {
                    println!("{}: not found", args);
                }
            }
            "pwd" => {
                let path = env::current_dir().unwrap();
                println!("{}", path.to_string_lossy());
            }
            "cd" => {
                if env::set_current_dir(args).is_ok() {
                    println!("{}", args);
                } else {
                    println!("{}: {}: No such file or directory", cmd, args);
                }
            }
            rest => {
                if is_valid_cmd(rest).is_some() {
                    exe_cmd(&rest, args)
                } else {
                    println!("{}: command not found", rest.trim());
                }
            }
        }
    }
}

fn exe_cmd(path: &str, args: &str) {
    let mut command = Command::new(path);
    for arg in args.split_whitespace() {
        command.arg(arg);
    }
    let output = command.output().unwrap();
    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else {
        println!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
}

fn is_valid_cmd(cmd: &str) -> Option<String> {
    let path_var = env::var_os("PATH").unwrap();
    let target_name = if cfg!(windows) {
        format!("{}.exe", cmd)
    } else {
        cmd.to_string()
    };
    for mut path in env::split_paths(&path_var) {
        path.push(&target_name);
        if path.is_file()
            && let Ok(metadata) = fs::metadata(&path)
            && metadata.is_file()
            && (metadata.permissions().mode() & 0o111) != 0
        {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}
