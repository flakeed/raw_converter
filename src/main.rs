use quickcheck::quickcheck;
use bitvec::prelude::*;
use std::{iter};
use bitvec::mem;
use bitvec::slice::BitSlice;
use criterion::{criterion_group, criterion_main, Criterion};


fn split_bytes_into_7bit_chunks<T:BitStore>(slice: &[u8]) -> (Vec<T>,usize) {
    (BitSlice::<_, Lsb0>::from_slice(slice)
        .chunks(mem::bits_of::<T>()-1)
        .flat_map(|chunk| chunk.iter().by_vals().chain(iter::once(false)))
        .collect::<BitVec<_>>()
        .into_vec(), slice.len())
}

fn join_7bit_chunks_into_bytes<T: BitStore>(chunks: &[T], bytes: usize) -> Vec<u8> {
    BitVec::<_, Lsb0>::from_slice(chunks)
        .chunks(mem::bits_of::<T>())
        .flat_map(|chunk| chunk.iter().take(chunk.len() - 1))
        .take(bytes*8)
        .collect::<BitVec<_>>()
        .into_vec()
}

fn ssplit_bytes_into_7bit_chunks(bytes: &[u8]) -> Vec<u8> {
    let mut chunks = Vec::new();
    let mut buffer: u16 = 0;
    let mut buffer_size = 0;
    let num_bytes = bytes.len();

    chunks.push(num_bytes as u8);

    for byte in bytes {
        buffer |= (*byte as u16) << buffer_size;
        buffer_size += 8;

        while buffer_size >= 7 {
            chunks.push((buffer & 0b0111_1111) as u8);
            buffer >>= 7;
            buffer_size -= 7;
        }
    }
    // If there are remaining bits in the buffer, add a final chunk
    if buffer_size > 0 {
        chunks.push((buffer & 0b0111_1111) as u8);
    }

    chunks
}

fn jjoin_7bit_chunks_into_bytes(chunks: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut buffer: u16 = 0;
    let mut buffer_size = 0;
    let num_bytes = chunks[0] as usize;

    for chunk in &chunks[1..] {
        buffer |= (*chunk as u16) << buffer_size;
        buffer_size += 7;

        while buffer_size >= 8 {
            bytes.push((buffer & 0xff) as u8);
            buffer >>= 8;
            buffer_size -= 8;
        }
    }

// If there are remaining bits in the buffer, add a final byte only if it's complete
    if buffer_size > 0 && buffer_size % 8 == 0 {
        bytes.push((buffer & 0xff) as u8);
    }
// Truncate the result to the original number of bytes
    bytes.truncate(num_bytes);

    bytes
}
quickcheck! {
        fn teest_old_algo(bytes: Vec<u8>) -> bool {
            let chunks = ssplit_bytes_into_7bit_chunks(&bytes);
            let reconstructed_bytes = jjoin_7bit_chunks_into_bytes(&chunks);
            bytes == reconstructed_bytes
        }
    }

quickcheck! {
    fn teest_new_algo(slice: Vec<u8>) -> bool {
        let (chunks, bytes_count) = split_bytes_into_7bit_chunks::<u8>(&slice);
        let reconstructed_slice = join_7bit_chunks_into_bytes(&chunks, bytes_count);
        slice == reconstructed_slice
    }
  }

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("test_old_algo", |b| {
        let bytes = (0..10000).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        b.iter(|| {
            let chunks = ssplit_bytes_into_7bit_chunks(&bytes);
            let reconstructed_bytes = jjoin_7bit_chunks_into_bytes(&chunks);
            assert_eq!(bytes, reconstructed_bytes);
        });
    });

    c.bench_function("test_new_algo", |b| {
        let slice = (0..10000).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        b.iter(|| {
            let (chunks, bytes_count) = split_bytes_into_7bit_chunks::<u8>(&slice);
            let reconstructed_slice = join_7bit_chunks_into_bytes(&chunks, bytes_count);
            assert_eq!(slice, reconstructed_slice);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);


