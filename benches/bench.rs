use criterion::Criterion;
use raw_converter::old;

fn criterion_benchmark(c: &mut Criterion) {
    #[repr(align(64))]
    struct Align<const N: usize>([u8; N]);

    let bytes = Align([243; 100]);
    let bytes = &bytes.0[1..];

    c.bench_function("old", |b| {
        b.iter(|| old::split_chunks(&bytes));
    });

    c.bench_function("new(u8)", |b| {
        b.iter(|| {
            let (_chunks, _valid_up) = split_into::<u8>(&bytes);
            //bytes == join_chunks(&chunks, valid_up)
        });
    });

    c.bench_function("new(u16)", |b| {
        b.iter(|| {
            let (_chunks, _valid_up) = split_into::<u16>(&bytes);
            //bytes == join_chunks(&chunks, valid_up)
        });
    });

    c.bench_function("new(u32)", |b| {
        b.iter(|| {
            let (_chunks, _valid_up) = split_into::<u32>(&bytes);
            //bytes == join_chunks(&chunks, valid_up)
        });
    });

    c.bench_function("new(u64)", |b| {
        b.iter(|| {
            let (_chunks, _valid_up) = split_into::<u64>(&bytes);
            //bytes == join_chunks(&chunks, valid_up)
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
