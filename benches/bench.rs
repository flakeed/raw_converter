#![feature(pointer_is_aligned)]

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
    PlottingBackend, Throughput,
};
use raw_converter::{old, split_into, Store};

fn define<T: Store>(group: &mut BenchmarkGroup<WallTime>, name: &str, corpus: &[u8]) {
    group.bench_function(name, |b| b.iter(|| split_into::<T>(corpus)));
}

fn with_data(c: &mut Criterion, name: &str, corpus: &[u8]) {
    let mut group = c.benchmark_group(&format!("split/{name}"));
    group.throughput(Throughput::BytesDecimal(corpus.len() as u64));

    group.bench_function("old", |b| {
        b.iter(|| old::split_chunks(corpus));
    });

    define::<u8>(&mut group, "new(u8)", corpus);
    define::<u16>(&mut group, "new(u16)", corpus);
    define::<u32>(&mut group, "new(u32)", corpus);
    define::<u64>(&mut group, "new(u64)", corpus);
}

const HUGE: &[u8] = include_bytes!("data/huge.txt");
const SMALL: &[u8] = include_bytes!("data/small.txt");
const TINY: &[u8] = include_bytes!("data/tiny.txt");

const THIN: &[u8] = &[1, 2, 3, 4];

fn all(c: &mut Criterion) {
    with_data(c, "huge", HUGE);
    with_data(c, "small", SMALL);
    with_data(c, "tiny", TINY);
    with_data(c, "thin", THIN);

    #[repr(align(64))]
    struct Align<const N: usize>([u8; N]);

    let bytes = Align([243u8; 128]);
    with_data(c, "unaligned", &bytes.0[1..]);

    let bytes = Align([243u8; 1024 * 128]);
    with_data(c, "unaligned-huge", &bytes.0[1..]);
}

criterion_group!(
    name = benches;
    config = Criterion::default().plotting_backend(PlottingBackend::Plotters);
    targets = all
);
criterion_main!(benches);
