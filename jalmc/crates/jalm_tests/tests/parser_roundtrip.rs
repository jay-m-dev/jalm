use jalm_tests::{diagnostics_json, round_trip};
use insta::{assert_json_snapshot, assert_snapshot};
use jalm_formatter::format_source;

#[test]
fn round_trip_snapshot_basic() {
    let src = "fn f()->i64{a+b}";
    let (lossless, tree) = round_trip(src);
    assert_eq!(lossless, src);
    assert_snapshot!(tree, @r###"
Root
  FnDecl
    KwFn 'fn'
    Whitespace ' '
    IdentNode
      Ident 'f'
    LParen '('
    ParamList
      RParen ')'
    Arrow '->'
    Type
      IdentNode
        Ident 'i64'
    Block
      LBrace '{'
      StmtList
        BinExpr
          IdentNode
            Ident 'a'
          Plus '+'
          IdentNode
            Ident 'b'
      RBrace '}'
"###);
}

#[test]
fn round_trip_expressions() {
    let src = "fn f()->i64{if true{foo(1+2).bar}else{match x{1=>2,_=>3,}}}";
    let (lossless, _tree) = round_trip(src);
    assert_eq!(lossless, src);
}

#[test]
fn round_trip_items() {
    let src = "mod foo;use crate::foo::bar as baz;struct User{id:i64;}enum Opt{Some(i64);None;}";
    let (lossless, _tree) = round_trip(src);
    assert_eq!(lossless, src);
}

#[test]
fn round_trip_whitespace_comments() {
    let src = "fn f(a: i64) -> i64 { /*c*/ let x = 1; x }";
    let (lossless, _tree) = round_trip(src);
    assert_eq!(lossless, src);
}

#[test]
fn diagnostics_missing_tokens() {
    let src = "fn f()->i64{let x=1}";
    let diags = diagnostics_json(src);
    assert_json_snapshot!(diags, @r###"
{
  "errors": [
    {
      "message": "expected Semi",
      "span": {
        "end": 20,
        "start": 19
      }
    },
    {
      "message": "expected RBrace",
      "span": {
        "end": 20,
        "start": 20
      }
    }
  ]
}
"###);
}

#[test]
fn diagnostics_bad_tokens() {
    let src = "fn f()->i64{let x=@}";
    let diags = diagnostics_json(src);
    assert_json_snapshot!(diags, @r###"
{
  "errors": [
    {
      "message": "expected expression",
      "span": {
        "end": 19,
        "start": 18
      }
    },
    {
      "message": "expected Semi",
      "span": {
        "end": 20,
        "start": 19
      }
    },
    {
      "message": "expected RBrace",
      "span": {
        "end": 20,
        "start": 20
      }
    }
  ]
}
"###);
}

#[test]
fn diagnostics_missing_rbrace() {
    let src = "fn f()->i64{let x=1;";
    let diags = diagnostics_json(src);
    assert_json_snapshot!(diags, @r###"
{
  "errors": [
    {
      "message": "expected RBrace",
      "span": {
        "end": 20,
        "start": 20
      }
    }
  ]
}
"###);
}

#[test]
fn formatter_idempotent() {
    let src = "fn f(a:i64)->i64{let x=1+2;return x;}";
    let once = format_source(src).expect("format once");
    let twice = format_source(&once).expect("format twice");
    assert_eq!(once, twice);
}

#[test]
fn formatter_normalizes_spacing() {
    let src = "fn f(a:i64)->i64{if true{foo(1+2).bar}else{match x{1=>2,_=>3,}}}";
    let formatted = format_source(src).expect("format");
    assert_snapshot!(formatted, @r###"
fn f(a: i64) -> i64 {
  if true {
    foo(1 + 2).bar
  } else {
    match x {
      1 => 2,
      _ => 3,
    }
  }
}
"###);
}
