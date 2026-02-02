use jalm_parser::parse;
use jalm_syntax::{dump_tree, to_string_lossless};
use serde_json::json;

pub fn round_trip(source: &str) -> (String, String) {
    let parsed = parse(source);
    let syntax = parsed.syntax();
    let lossless = to_string_lossless(&syntax);
    let tree = dump_tree(&syntax);
    (lossless, tree)
}

pub fn diagnostics_json(source: &str) -> serde_json::Value {
    let parsed = parse(source);
    json!({
        "errors": parsed.errors,
    })
}
