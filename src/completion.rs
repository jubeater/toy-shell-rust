use crate::{builtin, executable};

use rustyline::Context;
use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

pub struct ShellHelper;

impl ShellHelper {
    pub fn new() -> Self {
        Self
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Highlighter for ShellHelper {}

impl Validator for ShellHelper {}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let before_cursor = &line[..pos];
        let command_start = before_cursor.len() - before_cursor.trim_start().len();
        let prefix = &before_cursor[command_start..];

        // Arguments and explicit paths use filename completion.
        let (start, names) = if prefix.chars().any(char::is_whitespace) || prefix.contains('/') {
            let start = before_cursor
                .char_indices()
                .rev()
                .find(|(_, ch)| ch.is_whitespace())
                .map_or(0, |(index, ch)| index + ch.len_utf8());

            let filename_prefix = &line[start..pos];

            (start, complete_filenames(filename_prefix)?)
        } else {
            (command_start, complete_commands(prefix))
        };

        let unique = names.len() == 1;
        let candidates = names
            .into_iter()
            .map(|name| Pair {
                replacement: if unique && !name.ends_with('/') {
                    format!("{name} ")
                } else {
                    name.clone()
                },
                display: name,
            })
            .collect();

        Ok((start, candidates))
    }
}

fn complete_filenames(prefix: &str) -> std::io::Result<Vec<String>> {
    let mut matches = Vec::new();
    // Split from the right, maximum 2 parts
    let (path_prefix, filename_prefix) = match prefix.rfind('/') {
        Some(index) => prefix.split_at(index + 1),
        None => ("", prefix),
    };

    let search_directory = if path_prefix.is_empty() {
        "."
    } else {
        path_prefix
    };

    for entry in std::fs::read_dir(search_directory)?.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };

        if !name.starts_with(filename_prefix) {
            continue;
        }

        let Ok(metadata) = std::fs::metadata(entry.path()) else {
            continue;
        };

        if !metadata.is_file() && !metadata.is_dir() {
            continue;
        }

        let mut candidate = format!("{path_prefix}{name}");

        if metadata.is_dir() {
            candidate.push('/');
        }

        matches.push(candidate);
    }

    matches.sort();
    Ok(matches)
}

fn complete_commands(prefix: &str) -> Vec<String> {
    let mut matches: Vec<String> = builtin::Builtin::complete(prefix)
        .into_iter()
        .map(str::to_owned)
        .collect();

    matches.extend(executable::complete(prefix));
    matches.sort();
    matches.dedup();

    matches
}
