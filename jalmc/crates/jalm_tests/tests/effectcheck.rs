use insta::assert_json_snapshot;
use jalm_effectcheck::check;

#[test]
fn effectcheck_ok() {
    let src = "fn f() -> i64 !{fs} { fs::read(path); 1 }";
    let diags = check(src).diagnostics;
    assert!(diags.is_empty());
}

#[test]
fn effectcheck_missing_fs() {
    let src = "fn f() -> i64 { fs::read(path); 1 }";
    let diags = check(src).diagnostics;
    assert_json_snapshot!(diags, @r###"
[
  {
    "code": "E1001",
    "message": "undeclared effect",
    "span": {
      "start": 16,
      "end": 20
    },
    "required": "fs"
  }
]
"###);
}

#[test]
fn effectcheck_missing_net_http() {
    let src = "fn f() -> i64 { http::serve(); 1 }";
    let diags = check(src).diagnostics;
    assert_json_snapshot!(diags, @r###"
[
  {
    "code": "E1001",
    "message": "undeclared effect",
    "span": {
      "start": 16,
      "end": 22
    },
    "required": "net"
  }
]
"###);
}
