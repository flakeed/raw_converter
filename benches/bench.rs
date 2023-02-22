#![feature(pointer_is_aligned)]

use criterion::{criterion_group, criterion_main, Criterion};
use raw_converter::{old, split_into, Store};

#[repr(align(64))]
struct Align<const N: usize>([u8; N]);

fn criterion_benchmark(c: &mut Criterion) {
    let bytes = Align([243; 100]);
    let bytes = &bytes.0[2..];

    c.bench_function("old", |b| {
        b.iter(|| old::split_chunks(&bytes));
    });

    c.bench_function("new(u8)", |b| {
        b.iter(|| {
            split_into::<u8>(&bytes);
        });
    });

    c.bench_function("new(u16)", |b| {
        b.iter(|| {
            split_into::<u16>(&bytes);
        });
    });

    c.bench_function("new(u32)", |b| {
        b.iter(|| {
            split_into::<u32>(&bytes);
        });
    });

    c.bench_function("new(u64)", |b| {
        b.iter(|| {
            split_into::<u64>(&bytes);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
