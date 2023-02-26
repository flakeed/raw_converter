#![feature(pointer_is_aligned)]

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
    PlottingBackend, Throughput,
};
use raw_converter::{old, split_into, Store};

#[repr(align(64))]
struct Align<const N: usize>([u8; N]);

fn define<T: Store>(group: &mut BenchmarkGroup<WallTime>, name: &str, corpus: &[u8]) {
    group.bench_function(name, |b| b.iter(|| split_into::<T>(corpus)));
}

fn criterion_benchmark(c: &mut Criterion) {
    let bytes = Align([243; 1024 * 128]);
    let bytes = &bytes.0[1..];

    let mut group = c.benchmark_group("split");
    group.throughput(Throughput::BytesDecimal(bytes.len() as u64));

    group.bench_function("old", |b| {
        b.iter(|| old::split_chunks(bytes));
    });

    define::<u8>(&mut group, "new(u8)", bytes);
    define::<u16>(&mut group, "new(u16)", bytes);
    define::<u32>(&mut group, "new(u32)", bytes);
    define::<u64>(&mut group, "new(u64)", bytes);
}

criterion_group!(
    name = benches;
    config = Criterion::default().plotting_backend(PlottingBackend::Plotters);
    targets = criterion_benchmark
);
criterion_main!(benches);
