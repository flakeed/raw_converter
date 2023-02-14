use quickcheck::quickcheck;
use bitvec::prelude::*;
use std::{iter};
use bitvec::mem;
use bitvec::slice::BitSlice;

fn split_bytes_into_7bit_chunks<T:BitStore>(slice: &[u8]) -> (Vec<T>,usize) {
    let chunked = BitSlice::<_, Lsb0>::from_slice(slice)
        .chunks(mem::bits_of::<T>()-1)
        .flat_map(|chunk| chunk.iter().by_vals().chain(iter::once(false)))
        .collect::<BitVec<_>>()
        .into_vec();
    let bytes_count = slice.len();
    (chunked, bytes_count)
}

fn join_7bit_chunks_into_bytes<T: BitStore>(chunks: &[T], bytes: usize) -> Vec<u8> {
    BitVec::<_, Lsb0>::from_slice(chunks)
        .chunks(mem::bits_of::<T>())
        .flat_map(|chunk| chunk.iter().take(chunk.len() - 1))
        .take(bytes*8)
        .collect::<BitVec<_>>()
        .into_vec()
}

quickcheck! {
    fn prop_u8(slice: Vec<u8>) -> bool {
        let (chunks, bytes_count) = split_bytes_into_7bit_chunks::<u8>(&slice);
        let reconstructed_slice = join_7bit_chunks_into_bytes(&chunks, bytes_count);
        slice == reconstructed_slice
    }
    fn prop_u16(slice: Vec<u8>) -> bool {
        let (chunks, bytes_count) = split_bytes_into_7bit_chunks::<u16>(&slice);
        let reconstructed_slice = join_7bit_chunks_into_bytes(&chunks, bytes_count);
        slice == reconstructed_slice
    }
    fn prop_u32(slice: Vec<u8>) -> bool {
        let (chunks, bytes_count) = split_bytes_into_7bit_chunks::<u32>(&slice);
        let reconstructed_slice = join_7bit_chunks_into_bytes(&chunks, bytes_count);
        slice == reconstructed_slice
    }
    fn prop_u64(slice: Vec<u8>) -> bool {
        let (chunks, bytes_count) = split_bytes_into_7bit_chunks::<u64>(&slice);
        let reconstructed_slice = join_7bit_chunks_into_bytes(&chunks, bytes_count);
        slice == reconstructed_slice
    }
  }

fn main(){

}

