use std::env;
#[allow(unused_imports)]
use std::fs;

use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "exit" => {
                break;
            }
            e if e.starts_with("echo") => {
                println!("{}", &e[5..]);
            }
            e if e.starts_with("type") => {
                if let Some(cmd_path) = is_valid_cmd(&e[5..]) {
                    println!("{} is {}", &e[5..], cmd_path);
                } else {
                    println!("{}: not found", &e[5..]);
                }
            }
            cmd => {
                println!("{}: command not found", cmd);
            }
        }
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
