use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

use rayon::prelude::*;
use regex::Regex;
use sprinkles::{buckets::Bucket, contexts::User, packages::SearchMode};

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = User::new().unwrap();
    // let all_buckets = Bucket::list_all().unwrap();

    let pattern = Regex::new("(?i)google").unwrap();

    c.bench_function("list buckets", |b| {
        b.iter(|| Bucket::list_all(&ctx).unwrap())
    });

    c.bench_function("match packages", |b| {
        b.iter(|| {
            black_box(Bucket::list_all(&ctx).unwrap())
                .par_iter()
                .filter_map(|bucket| {
                    match bucket.matches(&ctx, false, &pattern, black_box(SearchMode::Name)) {
                        Ok(section) => Some(section),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("listing packages unchecked", |b| {
        for bucket in Bucket::list_all(&ctx).unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| bucket.list_packages_unchecked(),
                BatchSize::SmallInput,
            )
        }
    });

    c.bench_function("listing packages", |b| {
        for bucket in Bucket::list_all(&ctx).unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| bucket.list_packages(),
                BatchSize::SmallInput,
            )
        }
    });

    c.bench_function("listing packages from names", |b| {
        for bucket in Bucket::list_all(&ctx).unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| bucket.list_package_paths().unwrap(),
                BatchSize::SmallInput,
            )
        }
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
