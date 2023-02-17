use criterion::{criterion_group, criterion_main, Criterion};
use my_crate::{split_bytes_into_7bit_chunks, join_7bit_chunks_into_bytes, ssplit_bytes_into_7bit_chunks, jjoin_7bit_chunks_into_bytes};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("test_old_algo", |b| {
        let bytes = (0..10000).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        b.iter(|| {
            let chunks = ssplit_bytes_into_7bit_chunks(&bytes);
            let reconstructed_bytes = jjoin_7bit_chunks_into_bytes(&chunks);
            bytes == reconstructed_bytes
        });
    });

    c.bench_function("test_new_algo", |b| {
        let slice = (0..10000).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        b.iter(|| {
            let (chunks, bytes_count) = split_bytes_into_7bit_chunks::<u8>(&slice);
            let reconstructed_slice = join_7bit_chunks_into_bytes(&chunks, bytes_count);
            slice == reconstructed_slice
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
