// parser.rs

pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub stdout: Option<Redirect>,
    pub stderr: Option<Redirect>,
}

pub enum Redirect {
    Truncate(String),
    Append(String),
}

pub enum ParseError {
    NoCmd,
    MissingFileName(String),
}
