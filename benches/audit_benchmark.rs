//! Benchmarks de auditoria

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn audit_benchmark(c: &mut Criterion) {
    c.bench_function("audit_stub", |b| {
        b.iter(|| black_box(42))
    });
}

criterion_group!(benches, audit_benchmark);
criterion_main!(benches);
