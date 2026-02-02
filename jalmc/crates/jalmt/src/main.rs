use jalm_parser::parse;
use serde_json::json;
use std::env;
use std::fs;

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 || args[0] != "parse" {
        eprintln!("usage: jalmt parse <file>");
        std::process::exit(2);
    }
    let path = args.remove(1);
    let source = match fs::read_to_string(&path) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("failed to read {}: {}", path, err);
            std::process::exit(1);
        }
    };

    let parsed = parse(&source);
    let diag = json!({
        "errors": parsed.errors,
    });
    println!("{}", serde_json::to_string_pretty(&diag).unwrap());
}
