#![feature(pointer_is_aligned)]

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
    PlottingBackend, Throughput,
};
use raw_converter::{join_into, old, split_into, Store};

fn define<T: Store>(group: &mut BenchmarkGroup<WallTime>, name: &str, corpus: &[u8]) {
    group.bench_function(name, |b| {
        b.iter(|| {
            let (res, meta) = split_into::<T>(corpus);
            assert_eq!(join_into(&res, meta), corpus);
        })
    });
}

fn with_data(c: &mut Criterion, name: &str, corpus: &[u8]) {
    let mut group = c.benchmark_group(name);
    group
        // -- for fast dev rebench --
        // .sample_size(10)
        // .warm_up_time(std::time::Duration::from_millis(500))
        // .measurement_time(std::time::Duration::from_secs(2))
        // --                      --
        .throughput(Throughput::BytesDecimal(corpus.len() as u64));

    group.bench_function("old", |b| {
        b.iter(|| {
            let (chunks, len) = old::split_chunks(&corpus);
            assert_eq!(corpus, old::join_chunks(&chunks, len));
        });
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
const EMPTY: &[u8] = &[];

fn all(c: &mut Criterion) {
    with_data(c, "huge", HUGE);
    with_data(c, "small", SMALL);
    with_data(c, "tiny", TINY);
    with_data(c, "thin", THIN);
    with_data(c, "empty", EMPTY);
}

criterion_group!(
    name = benches;
    config = Criterion::default().plotting_backend(PlottingBackend::Plotters);
    targets = all
);
criterion_main!(benches);
