use jalm_formatter::format_source;
use jalm_parser::parse;
use serde_json::json;
use std::env;
use std::fs;

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!("usage: jalmt <parse|fmt> <file>");
        std::process::exit(2);
    }
    let cmd = args.remove(0);
    if args.is_empty() {
        eprintln!("usage: jalmt {} <file>", cmd);
        std::process::exit(2);
    }
    let path = args.remove(0);
    let source = match fs::read_to_string(&path) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("failed to read {}: {}", path, err);
            std::process::exit(1);
        }
    };

    match cmd.as_str() {
        "parse" => {
            let parsed = parse(&source);
            let diag = json!({
                "errors": parsed.errors,
            });
            println!("{}", serde_json::to_string_pretty(&diag).unwrap());
        }
        "fmt" => match format_source(&source) {
            Ok(formatted) => {
                if formatted != source {
                    if let Err(err) = fs::write(&path, formatted) {
                        eprintln!("failed to write {}: {}", path, err);
                        std::process::exit(1);
                    }
                }
            }
            Err(err) => {
                eprintln!("format error: {:?}", err);
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!("usage: jalmt <parse|fmt> <file>");
            std::process::exit(2);
        }
    }
}
