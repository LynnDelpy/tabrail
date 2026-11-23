//! A tool call as one line: the sidebar's icon for the tool, its name and a short form of its
//! argument (the command, the file with ~ for $HOME, the pattern, the skill, else the first string).
use crate::json::Value;

pub fn line(name: &str, input: &Value) -> String {
    let icon = match name {
        "Bash" => '⚙',
        "Read" => '◎',
        "Grep" | "Glob" => '⌕',
        "Edit" | "Write" => '✎',
        "Task" | "Agent" => '⧉',
        _ => '·',
    };
    let first = match input {
        Value::Obj(fields) => fields.iter().find_map(|(_, v)| v.str()),
        _ => None,
    };
    let field = |key: &str| input.get(key).str();
    let arg = field("command").map(String::from)
        .or_else(|| field("file_path").map(home))
        .or_else(|| field("pattern").or(field("skill")).or(first).map(String::from))
        .unwrap_or_default();
    // one row: a heredoc's lines and runs of spaces as single spaces
    format!("{icon} {name} {}", arg.split_whitespace().collect::<Vec<_>>().join(" ")).trim_end().into()
}

fn home(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(h) if h.len() > 1 && (path == h || path.starts_with(&format!("{h}/"))) => format!("~{}", &path[h.len()..]),
        _ => path.into(),
    }
}
