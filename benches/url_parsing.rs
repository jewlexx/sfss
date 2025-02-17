use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse url", |b| {
        b.iter_batched(
            || "https://github.com/winpax/sfsu/releases/download/v1.17.0/sfsu-1.17.0-x86_64.msi",
            |url| black_box(url::Url::parse(black_box(url)).unwrap()),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
