use std::str::FromStr;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

use sprinkles::{contexts::User, packages::reference::package};

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = User::new().unwrap();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("find package across buckets", "sfsu"),
        &package::Reference::from_str("sfsu").unwrap(),
        |b, package| {
            b.to_async(&runtime)
                .iter(|| black_box(package.manifest(&ctx)))
        },
    );

    c.bench_with_input(
        BenchmarkId::new("find package with version across buckets", "sfsu@1.10.0"),
        &package::Reference::from_str("sfsu@1.10.0").unwrap(),
        |b, package| {
            b.to_async(&runtime)
                .iter(|| black_box(package.manifest(&ctx)))
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
