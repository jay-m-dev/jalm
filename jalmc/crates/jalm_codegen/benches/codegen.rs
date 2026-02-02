use criterion::{criterion_group, criterion_main, Criterion};
use jalm_codegen::compile_to_wasm;

fn bench_compile_to_wasm(c: &mut Criterion) {
    let source = r#"
fn add(a: i64, b: i64) -> i64 {
  return a + b;
}

fn main() -> i64 {
  let x: i64 = add(10, 32);
  return x + 1;
}
"#;

    c.bench_function("compile_to_wasm_small", |b| {
        b.iter(|| {
            let _ = compile_to_wasm(source).expect("compile ok");
        })
    });
}

criterion_group!(benches, bench_compile_to_wasm);
criterion_main!(benches);
