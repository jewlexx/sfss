use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use sprinkles::{contexts::User, packages::reference::Package};

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = User::new();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("find package across buckets", "sfsu"),
        &Package::from_str("sfsu").unwrap(),
        |b, package| {
            b.to_async(&runtime)
                .iter(|| black_box(package.manifest(&ctx)))
        },
    );

    c.bench_with_input(
        BenchmarkId::new("find package with version across buckets", "sfsu@1.10.0"),
        &Package::from_str("sfsu@1.10.0").unwrap(),
        |b, package| {
            b.to_async(&runtime)
                .iter(|| black_box(package.manifest(&ctx)))
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
