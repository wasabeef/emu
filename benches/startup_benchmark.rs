use criterion::{black_box, criterion_group, criterion_main, Criterion};
use emu::app::App;
use emu::config::Config;

fn startup_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("app_startup", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = black_box(Config::default());
                let app = App::new(config).await.unwrap();
                black_box(app)
            })
        })
    });
}

fn config_creation_benchmark(c: &mut Criterion) {
    c.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = black_box(Config::default());
            black_box(config)
        })
    });
}

criterion_group!(benches, startup_benchmark, config_creation_benchmark);
criterion_main!(benches);
