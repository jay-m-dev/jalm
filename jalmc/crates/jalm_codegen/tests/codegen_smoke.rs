use jalm_codegen::compile_to_wasm;
use wasmtime::{Engine, Instance, Module, Store};

fn run_main(source: &str) -> i64 {
    let wasm = compile_to_wasm(source).expect("compile ok");
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).expect("wasm module");
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("instance");
    let main = instance
        .get_typed_func::<(), i64>(&mut store, "main")
        .expect("main func");
    main.call(&mut store, ()).expect("call main")
}

#[test]
fn compile_and_run_simple_main() {
    let source = r#"
fn main() -> i64 {
  return 42;
}
"#;
    assert_eq!(run_main(source), 42);
}

#[test]
fn compile_with_let_and_call() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 {
  return a + b;
}

fn main() -> i64 {
  let x: i64 = add(10, 32);
  return x;
}
"#;
    assert_eq!(run_main(source), 42);
}

#[test]
fn unknown_function_reports_error() {
    let source = r#"
fn main() -> i64 {
  return nope();
}
"#;
    let errs = compile_to_wasm(source).unwrap_err();
    assert!(errs.iter().any(|d| d.code == "E2005"));
}
