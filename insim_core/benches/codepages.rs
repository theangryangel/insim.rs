#![allow(missing_docs, missing_debug_implementations, unused_results)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use insim_core::string::codepages::{to_lossy_bytes, to_lossy_string};

fn bench_to_lossy_string(c: &mut Criterion) {
    let input: &[u8] = &[
        94, // ^
        69, // E
        248, 94, // ^
        67, // C
        248, 94, // ^
        76, // L
        248,
    ];

    c.bench_function("to_lossy_string", |b| {
        b.iter(|| to_lossy_string(black_box(input)))
    });
}

fn bench_to_lossy_bytes_ascii(c: &mut Criterion) {
    let input: &str = "Hello world!";

    c.bench_function("to_lossy_bytes", |b| {
        b.iter(|| to_lossy_bytes(black_box(input)))
    });
}

fn bench_to_lossy_bytes_simple(c: &mut Criterion) {
    let input: &str = "《TEST》Árvíztűrő tükörfúrógép";

    c.bench_function("to_lossy_bytes", |b| {
        b.iter(|| to_lossy_bytes(black_box(input)))
    });
}

fn bench_to_lossy_bytes_complex(c: &mut Criterion) {
    let input: &str = "Hello Привет こんにちは 世界 Árvíztűrő tükörfúrógép World!";

    c.bench_function("to_lossy_bytes", |b| {
        b.iter(|| to_lossy_bytes(black_box(input)))
    });
}

criterion_group!(
    benches,
    bench_to_lossy_string,
    bench_to_lossy_bytes_ascii,
    bench_to_lossy_bytes_simple,
    bench_to_lossy_bytes_complex
);
criterion_main!(benches);
