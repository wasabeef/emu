use criterion::{criterion_group, criterion_main, Criterion};

fn dummy_benchmark(_c: &mut Criterion) {
    // ここにベンチマークしたい処理を記述します。
    // 例: c.bench_function("fib 20", |b| b.iter(|| fibonacci(20)));
}

criterion_group!(benches, dummy_benchmark);
criterion_main!(benches);
