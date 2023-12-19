use std::{collections::VecDeque, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, LitStr, Token};

const EXAMPLES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
const RESERVED_COMMENTS: &[&str] = &[
    "begin-code-snippet",
    "end-code-snippet",
    "begin-hidden-code",
    "end-hidden-code",
    "begin-ellipses",
    "end-ellipses",
];

#[derive(Debug, Clone)]
struct GetSnippetsArgs {
    example: String,
    snippets: Vec<String>,
}

impl Parse for GetSnippetsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect::<VecDeque<_>>();
        Ok(Self {
            example: args
                .pop_front()
                .expect("expected first argument `example`")
                .value(),
            snippets: args.into_iter().map(|x| x.value()).collect(),
        })
    }
}

#[proc_macro]
pub fn get_snippets(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as GetSnippetsArgs);

    let path = PathBuf::from(EXAMPLES_DIR).join(format!("{}.rs", args.example));
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read example file: {path:?}: {e}"));

    let mut in_snippet = false;
    let mut hidden = false;
    let mut full = "```\n".to_string();
    let mut selected = String::new();
    let mut current_snippet = 0;

    for line in contents.split('\n') {
        let trimmed = line.trim();
        let trimmed_len = line.trim_start().len();
        if current_snippet < args.snippets.len() {
            if trimmed == format!("// begin-code-snippet {}", &args.snippets[current_snippet]) {
                in_snippet = true;
                hidden = false;
                continue;
            } else if trimmed == format!("// end-code-snippet {}", &args.snippets[current_snippet])
            {
                current_snippet += 1;
                in_snippet = false;
                if current_snippet < args.snippets.len() {
                    selected.push('\n');
                }

                full.push_str(&textwrap::dedent(&selected));
                selected = String::new();
                continue;
            } else if trimmed.starts_with("// begin-hidden-code") {
                hidden = true;
                continue;
            } else if trimmed.starts_with("// end-hidden-code") {
                hidden = false;
                continue;
            } else if in_snippet
                && trimmed == format!("// begin-ellipses {}", &args.snippets[current_snippet])
            {
                hidden = true;
                selected.push('\n');
                selected.push_str(&line[..line.len() - trimmed_len]);
                selected.push_str("// ...\n\n");
                continue;
            } else if trimmed == format!("// end-ellipses {}", &args.snippets[current_snippet]) {
                hidden = false;
                continue;
            } else if RESERVED_COMMENTS
                .iter()
                .map(|text| format!("// {}", text))
                .any(|comment| trimmed.starts_with(&comment))
            {
                continue;
            }
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

    if current_snippet < args.snippets.len() {
        panic!(
            "Code snippet {:?} not found",
            args.snippets[current_snippet]
        );
    }

    full.push_str("```");

    quote! {
        #full
    }
    .into()
}
