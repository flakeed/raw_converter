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
    let bytes_count = chunked.len();
    (chunked, bytes_count)
}

fn join_7bit_chunks_into_bytes<T: BitStore>(chunks: &[T], bytes: usize) -> Vec<u8> {
    BitVec::<_, Lsb0>::from_slice(chunks)
        .chunks(mem::bits_of::<T>())
        .flat_map(|chunk| chunk.iter().take(chunk.len() - 1))
        .take(bytes*8-8)
        .collect::<BitVec<_>>()
        .into_vec()
}

#[test]
fn test_split_bytes_into_7bit_chunks_u8() {
    let bytes = vec![0b1101_0101,0b1010_1010];
    let (chunks,remainder) = split_bytes_into_7bit_chunks::<u8>(&bytes);
    let joined_bytes=join_7bit_chunks_into_bytes(&chunks,remainder);
    assert_eq!(chunks, [0b0101_0101, 0b0101_0101, 0b0000_0010]);
    assert_eq!(bytes,joined_bytes);

    let bytes = vec![0b0111_1111, 0b1111_1111];
    let (chunks,remainder) = split_bytes_into_7bit_chunks::<u8>(&bytes);
    let joined_bytes=join_7bit_chunks_into_bytes(&chunks,remainder);
    assert_eq!(chunks, [0b0111_1111, 0b0111_1110, 0b0000_0011]);
    assert_eq!(bytes,joined_bytes);

    let bytes = vec![0b0111_1111, 0b1111_1111, 0b0111_1110];
    let (chunks,remainder) = split_bytes_into_7bit_chunks::<u8>(&bytes);
    let joined_bytes=join_7bit_chunks_into_bytes(&chunks,remainder);
    assert_eq!(chunks, [0b0111_1111, 0b0111_1110, 0b0111_1011, 0b0000_0011]);
    assert_eq!(bytes,joined_bytes);

    let bytes = vec![0b0000_0000];
    let (chunks,remainder) = split_bytes_into_7bit_chunks::<u8>(&bytes);
    let joined_bytes=join_7bit_chunks_into_bytes(&chunks,remainder);
    assert_eq!(chunks, [0b0000_0000, 0b0000_0000]);
    assert_eq!(bytes,joined_bytes);

    let bytes = vec![0b1000_0000];
    let (chunks,remainder) = split_bytes_into_7bit_chunks::<u8>(&bytes);
    let joined_bytes=join_7bit_chunks_into_bytes(&chunks,remainder);
    assert_eq!(chunks, [0b0000_0000, 0b0000_0001]);
    assert_eq!(bytes,joined_bytes);
}

fn main(){

}

