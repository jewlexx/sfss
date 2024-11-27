use std::{str::FromStr, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use sprinkles::{
    cache::{DownloadHandle, Handle},
    contexts::{ScoopContext, User},
    packages::reference::package,
    requests::{AsyncClient, Client},
    Architecture,
};

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = User::new().unwrap();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    c.bench_function("parse package", |b| {
        b.iter(|| black_box(package::Reference::from_str("extras/sfsu").unwrap()));
    });

    c.bench_function("get package manifest", |b| {
        b.to_async(&runtime).iter_batched(
            || {
                (
                    package::Reference::from_str("extras/sfsu").unwrap(),
                    ctx.clone(),
                )
            },
            |(package, ctx)| async move { black_box(package.manifest(&ctx).await.unwrap()) },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("create clients", |b| {
        b.iter(|| black_box(Client::blocking()));
    });

    c.bench_function("create async clients", |b| {
        b.iter(|| black_box(Client::asynchronous()));
    });

    c.bench_function("open handle", |b| {
        b.to_async(&runtime).iter_batched(
            || async {
                package::Reference::from_str("extras/sfsu")
                    .unwrap()
                    .manifest(&ctx)
                    .await
                    .unwrap()
            },
            |manifest| async {
                Handle::open_manifest(ctx.cache_path(), &manifest.await, Architecture::ARCH)
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    let mut slow = c.benchmark_group("updating manifest properties");
    slow.sample_size(10);
    slow.measurement_time(Duration::from_secs(10));

    slow.bench_function("set version", |b| {
        b.to_async(&runtime).iter_batched(
            || {
                (
                    package::Reference::from_str("extras/sfsu").unwrap(),
                    ctx.clone(),
                )
            },
            |(mut package, ctx)| async move {
                black_box(&mut package).set_version("1.10.2".to_string());
                black_box(package.manifest(&ctx).await.unwrap());
            },
            BatchSize::SmallInput,
        );
    });

    slow.bench_function("create downloader", |b| {
        b.to_async(&runtime).iter_batched(
            || async {
                Handle::open_manifest(
                    ctx.cache_path(),
                    &package::Reference::from_str("extras/sfsu")
                        .unwrap()
                        .manifest(&ctx)
                        .await
                        .unwrap(),
                    Architecture::ARCH,
                )
                .unwrap()
                .remove(0)
            },
            |dl| async {
                let dl = dl.await;
                black_box(
                    DownloadHandle::new::<AsyncClient>(dl, None, None)
                        .await
                        .unwrap(),
                )
            },
            BatchSize::SmallInput,
        );
    });

    slow.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
