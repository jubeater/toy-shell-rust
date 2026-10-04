use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

pub fn execute_external(cmd: &str, args: &str) {
    if find_executable(cmd).is_some() {
        exe_cmd(&cmd, args)
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
