use insta::assert_json_snapshot;
use jalm_typecheck::check;

#[test]
fn typecheck_ok() {
    let src = "fn add(a: i64, b: i64) -> i64 { let c = a + b; c }";
    let diags = check(src).diagnostics;
    assert!(diags.is_empty());
}

#[test]
fn typecheck_mismatch() {
    let src = "fn f(a: i64) -> bool { a + 1 }";
    let diags = check(src).diagnostics;
    assert_json_snapshot!(diags, @r###"
[
  {
    "code": "E0004",
    "message": "type mismatch",
    "span": {
      "start": 21,
      "end": 30
    },
    "expected": "bool",
    "actual": "i64"
  }
]
"###);
}

#[test]
fn typecheck_undefined_var() {
    let src = "fn f() -> i64 { x }";
    let diags = check(src).diagnostics;
    assert_json_snapshot!(diags, @r###"
[
  {
    "code": "E0001",
    "message": "undefined variable",
    "span": {
      "start": 16,
      "end": 17
    },
    "expected": null,
    "actual": "x"
  }
]
"###);
}
