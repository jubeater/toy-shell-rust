use crate::parser::Redirect;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

pub fn execute_external(
    cmd: &str,
    args: &[String],
    stdout: Option<&Redirect>,
    stderr: Option<&Redirect>,
) {
    if find_executable(cmd).is_some() {
        let mut command = Command::new(cmd);
        command.args(args);
        if let Some(redirect) = stdout {
            let file = match redirect {
                Redirect::Truncate(path) => File::create(path).unwrap(),
                Redirect::Append(path) => OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap(),
            };
            command.stdout(Stdio::from(file));
        }
        if let Some(redirect) = stderr {
            let file = match redirect {
                Redirect::Truncate(path) => File::create(path).unwrap(),
                Redirect::Append(path) => OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap(),
            };
            command.stderr(Stdio::from(file));
        }
        command.status().unwrap();
    } else {
        println!("{}: command not found", cmd.trim());
    }
}

pub fn find_executable(cmd: &str) -> Option<PathBuf> {
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
            return Some(path);
        }
    }
    None
}

pub fn complete(prefix: &str) -> Vec<String> {
    let Some(path_var) = env::var_os("PATH") else {
        return vec![];
    };

    let mut matches = Vec::new();

    for dir in env::split_paths(&path_var) {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            let Ok(metadata) = fs::metadata(&path) else {
                continue;
            };

            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }

            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if name.starts_with(prefix) {
                matches.push(name.to_string());
            }
        }
    }

    matches.sort();
    matches.dedup();

    matches
}
