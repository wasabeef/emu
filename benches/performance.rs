use criterion::{criterion_group, criterion_main, Criterion};

fn dummy_benchmark(_c: &mut Criterion) {
    // Write the processing you want to benchmark here.
    // Example: c.bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}

criterion_group!(benches, dummy_benchmark);
criterion_main!(benches);
