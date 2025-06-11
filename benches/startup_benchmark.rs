use criterion::{black_box, criterion_group, criterion_main, Criterion};
use emu::app::App;

fn startup_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("app_startup", |b| {
        b.iter(|| {
            rt.block_on(async {
                let app = App::new().await.unwrap();
                black_box(app)
            })
        })
    });
}

criterion_group!(benches, startup_benchmark);
criterion_main!(benches);
