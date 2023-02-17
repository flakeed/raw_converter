use quickcheck::quickcheck;
use my_crate::{split_bytes_into_7bit_chunks, join_7bit_chunks_into_bytes, ssplit_bytes_into_7bit_chunks, jjoin_7bit_chunks_into_bytes};

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

fn main(){

}

