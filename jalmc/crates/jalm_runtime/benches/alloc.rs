use criterion::{criterion_group, criterion_main, Criterion};
use jalm_runtime::{jalm_alloc, jalm_realloc};

fn bench_alloc(c: &mut Criterion) {
    c.bench_function("jalm_alloc_64", |b| {
        b.iter(|| {
            let ptr = jalm_alloc(64);
            if ptr.is_null() {
                panic!("alloc failed");
            }
        })
    });

    c.bench_function("jalm_realloc_128", |b| {
        b.iter(|| {
            let ptr = jalm_alloc(64);
            if ptr.is_null() {
                panic!("alloc failed");
            }
            let new_ptr = jalm_realloc(ptr, 64, 128);
            if new_ptr.is_null() {
                panic!("realloc failed");
            }
        })
    });
}

criterion_group!(benches, bench_alloc);
criterion_main!(benches);
