use std::env;
#[allow(unused_imports)]
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

        let (cmd, args) = input.split_once(' ').unwrap_or((&input, ""));
        match cmd {
            "exit" => {
                break;
            }
            e if e == "echo" => {
                println!("{}", args);
            }
            e if e == "type" => {
                const BUILDIN_CMDS: [&str; 3] = ["echo", "exit", "type"];
                if BUILDIN_CMDS.contains(&args) {
                    println!("{} is a shell builtin", args);
                } else if let Some(cmd_path) = is_valid_cmd(args) {
                    println!("{} is {}", args, cmd_path);
                } else {
                    println!("{}: not found", args);
                }
            }
            rest => {
                if is_valid_cmd(rest).is_some() {
                    exe_cmd(rest, args)
                } else {
                    println!("{}: command not found", rest.trim());
                }
            }
        }
    }
}

fn exe_cmd(path: &str, args: &str) {
    let mut command = Command::new(path);
    for arg in args.split(' ') {
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
