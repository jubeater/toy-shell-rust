use crate::parser::Redirect;
use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn execute_external(
    cmd: &str,
    args: &[String],
    stdout: Option<&Redirect>,
    stderr: Option<&Redirect>,
) -> io::Result<()> {
    if let Some(path) = find_executable(cmd) {
        let mut command = Command::new(cmd);
        command.args(args);
        if let Some(redirect) = stdout {
            let file = redirect.open()?;
            command.stdout(Stdio::from(file));
        }
        if let Some(redirect) = stderr {
            let file = redirect.open()?;
            command.stderr(Stdio::from(file));
        }
        command.status()?;
        Ok(())
    } else {
        println!("{}: command not found", cmd.trim());
        Ok(())
    }
}

pub fn find_executable(cmd: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for mut path in env::split_paths(&path_var) {
        path.push(cmd);
        if let Ok(metadata) = fs::metadata(&path)
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
