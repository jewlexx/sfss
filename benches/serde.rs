use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

use sprinkles::packages::Manifest;

fn criterion_benchmark(c: &mut Criterion) {
    const MANIFEST: &str = r#"{"version":"0.10.1","description":"General-purpose programming language designed for robustness, optimality, and maintainability.","homepage":"https://ziglang.org/","license":"MIT","suggest":{"vcredist":"extras/vcredist2022"},"architecture":{"64bit":{"url":"https://ziglang.org/download/0.10.1/zig-windows-x86_64-0.10.1.zip","hash":"5768004e5e274c7969c3892e891596e51c5df2b422d798865471e05049988125","extract_dir":"zig-windows-x86_64-0.10.1"},"arm64":{"url":"https://ziglang.org/download/0.10.1/zig-windows-aarch64-0.10.1.zip","hash":"ece93b0d77b2ab03c40db99ef7ccbc63e0b6bd658af12b97898960f621305428","extract_dir":"zig-windows-aarch64-0.10.1"}},"bin":"zig.exe","checkver":{"url":"https://ziglang.org/download/","regex":">([\\d.]+)</h"},"autoupdate":{"architecture":{"64bit":{"url":"https://ziglang.org/download/$version/zig-windows-x86_64-$version.zip","extract_dir":"zig-windows-x86_64-$version"},"arm64":{"url":"https://ziglang.org/download/$version/zig-windows-aarch64-$version.zip","extract_dir":"zig-windows-aarch64-$version"}},"hash":{"url":"https://ziglang.org/download/","regex":"(?sm)$basename.*?$sha256"}}}"#;

    let deserialized: Manifest = serde_json::from_str(MANIFEST).unwrap();

    c.bench_function("serialize struct to string", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&deserialized).unwrap());
        });
    });

    c.bench_function("serialize value to string", |b| {
        let value = serde_json::to_value(&deserialized).unwrap();

        b.iter(|| {
            black_box(serde_json::to_string(&value).unwrap());
        });
    });

    c.bench_function("serialize struct to string pretty", |b| {
        b.iter(|| {
            black_box(serde_json::to_string_pretty(&deserialized).unwrap());
        });
    });

    c.bench_function("serialize value to string pretty", |b| {
        let value = serde_json::to_value(&deserialized).unwrap();

        b.iter(|| {
            black_box(serde_json::to_string_pretty(&value).unwrap());
        });
    });

    c.bench_function("deserialize string to struct", |b| {
        b.iter(|| {
            black_box::<Manifest>(serde_json::from_str(MANIFEST).unwrap());
        });
    });

    c.bench_function("deserialize value to struct", |b| {
        let value = serde_json::to_value(&deserialized).unwrap();

        b.iter_batched(
            || value.clone(),
            |value| {
                black_box::<Manifest>(serde_json::from_value(value).unwrap());
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
