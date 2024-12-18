use std::{
    collections::{HashSet, VecDeque},
    path::{Path, PathBuf},
};

use regex::Regex;

const EXAMPLES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
const RESERVED_COMMENTS: &[&str] = &[
    "begin-code-snippet",
    "end-code-snippet",
    "begin-hidden-code",
    "end-hidden-code",
    "begin-ellipses",
    "end-ellipses",
];

#[macro_export]
macro_rules! include_snippet {
    ($prefix: tt, $snippet: tt) => {
        include_str!(concat!(
            env!("OUT_DIR"),
            concat!("/", $prefix, "/", $snippet)
        ))
    };
}

pub fn build_snippets(source: impl AsRef<Path>, prefix: impl AsRef<str>) {
    let source = source.as_ref();
    let prefix = prefix.as_ref();
    let re = Regex::new(r"^[a-zA-Z0-9_\-]+$").unwrap();
    assert!(re.is_match(prefix), "prefix must match regex {re:?}");
    let contents = std::fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("could not read source file: {source:?}: {e}"));

    let re = Regex::new(r"^\s*// begin-code-snippet ([a-zA-Z0-9_\-]+).*$").unwrap();
    let mut snippets = HashSet::new();
    for line in contents.split('\n') {
        if let Some(caps) = re.captures(line) {
            snippets.insert(caps.get(1).unwrap().as_str().to_string());
        }
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    for snippet in &snippets {
        let out_file = out_dir.join(prefix).join(snippet);
        std::fs::create_dir_all(out_file.parent().unwrap())
            .expect("failed to create parent directories for {out_file:?}");

        std::fs::write(&out_file, build_snippet(&contents, snippet))
            .expect(&format!("failed to write build snippet at {out_file:?}"));
    }
}

pub fn build_snippet(contents: impl AsRef<str>, snippet: impl AsRef<str>) -> String {
    let contents = contents.as_ref();
    let snippet = snippet.as_ref();

    let mut in_snippet = false;
    let mut hidden = false;
    let mut full = "```\n".to_string();
    let mut selected = String::new();
    let mut current_snippet = 0;

    for line in contents.split('\n') {
        let trimmed = line.trim();
        let trimmed_len = line.trim_start().len();
        if trimmed == format!("// begin-code-snippet {}", snippet) {
            if current_snippet > 0 {
                selected.push('\n');
            }
            in_snippet = true;
            hidden = false;
            continue;
        } else if trimmed == format!("// end-code-snippet {}", snippet) {
            current_snippet += 1;
            in_snippet = false;

            full.push_str(&textwrap::dedent(&selected));
            selected = String::new();
            continue;
        } else if trimmed.starts_with("// begin-hidden-code") {
            hidden = true;
            continue;
        } else if trimmed.starts_with("// end-hidden-code") {
            hidden = false;
            continue;
        } else if in_snippet && trimmed == format!("// begin-ellipses {}", snippet) {
            hidden = true;
            selected.push('\n');
            selected.push_str(&line[..line.len() - trimmed_len]);
            selected.push_str("// ...\n\n");
            continue;
        } else if trimmed == format!("// end-ellipses {}", snippet) {
            hidden = false;
            continue;
        } else if RESERVED_COMMENTS
            .iter()
            .map(|text| format!("// {}", text))
            .any(|comment| trimmed.starts_with(&comment))
        {
            continue;
        }
        if !in_snippet {
            full.push_str("# ");
            full.push_str(line);
            full.push('\n');
        } else {
            if hidden {
                selected.push_str("# ");
            }
            selected.push_str(line);
            selected.push('\n');
        }
    }

    if current_snippet == 0 {
        panic!("Code snippet {:?} not found", snippet);
    }

    full.push_str("```");

    full
}
