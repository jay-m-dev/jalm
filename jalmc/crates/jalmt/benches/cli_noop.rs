use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use tempfile::TempDir;

fn bench_parse_small(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("main.jalm");
    fs::write(&file, "fn main() -> i64 { return 0; }").unwrap();

    c.bench_function("jalmt_parse_small", |b| {
        b.iter(|| {
            let mut cmd = assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("jalmt"));
            cmd.arg("parse").arg(&file);
            cmd.assert().success();
        })
    });
}

criterion_group!(benches, bench_parse_small);
criterion_main!(benches);
