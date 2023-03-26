#![feature(portable_simd)]
#![feature(array_chunks)]

use bitvec::{mem::bits_of, prelude::*};
use std::{
    mem,
    simd::{Mask, Simd, SimdPartialEq, ToBitMask},
    slice,
};
use tap::Tap;

pub mod old {
    pub fn split_chunks(bytes: &[u8]) -> (Vec<u8>, usize) {
        let mut chunks = Vec::new();
        let mut buffer: u16 = 0;
        let mut buffer_size = 0;

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

        (chunks, bytes.len())
    }

    pub fn join_chunks(chunks: &[u8], len: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut buffer: u16 = 0;
        let mut buffer_size = 0;
        let num_bytes = len;

        for chunk in chunks {
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
}

pub unsafe trait Store: BitStore + Sized {
    type Wide;
    type Mask;

    const MASK: Self;

    fn fill_up(bytes: &[u8]) -> Self;
    fn fill_bits_up(slice: &[Self]) -> (Self::Wide, usize);

    fn write_pack(dst: &mut Vec<Self>, pack: Self::Wide, up: usize);

    fn match_mask(self, mask: Self) -> bool;
    fn packed_mask(this: Self::Wide, other: Self::Wide) -> Self::Mask;

    unsafe fn write_packed(dst: &mut Vec<Self>, src: &[Self::Wide]);
}

macro_rules! impl_store {
    (($ty:ty|$mty:ty) where wide: $wide:expr; mask: $mask:expr) => {
        type Wide = Simd<$ty, $wide>;
        type Mask = Mask<$mty, $wide>;

        const MASK: Self = $mask;

        fn fill_up(bytes: &[u8]) -> Self {
            let mut place = [0u8; mem::size_of::<Self>()];

            unsafe {
                place[..bytes.len()].copy_from_slice(bytes);
                mem::transmute(place)
            }
        }

        fn fill_bits_up(slice: &[Self]) -> (Self::Wide, usize) {
            assert!(slice.len() < bits_of::<Self::Wide>());

            let mut place = Self::Wide::splat(0 as $ty);
            let bits = BitSlice::<_, Lsb0>::from_slice_mut(&mut place[..]);

            for idx in 0..slice.len() {
                // SAFETY: slice len is guarantee less than bits of `T`
                unsafe { bits.set_unchecked(idx, false) }
            }

            (place, again_aligned(slice.len(), bits_of::<Self>()))
        }

        fn write_pack(dst: &mut Vec<Self>, pack: Self::Wide, up: usize) {
            for src in &pack[..up] {
                dst.push(*src);
            }
        }

        fn match_mask(self, mask: Self) -> bool {
            self & mask != 0
        }

        fn packed_mask(this: Self::Wide, other: Self::Wide) -> Self::Mask {
            (this & other).simd_ne(Self::Wide::splat(0))
        }
    };
}

unsafe impl Store for u8 {
    impl_store!(
        (u8|i8) where wide: 64; mask: 1u8 << 7
    );

    unsafe fn write_packed(dst: &mut Vec<Self>, src: &[Self::Wide]) {
        for chunk in src {
            let bytes =
                Self::packed_mask(*chunk, Self::Wide::splat(Self::MASK)).to_bitmask().to_ne_bytes();
            dst.extend_from_slice(&bytes);
        }
    }
}

unsafe impl Store for u16 {
    impl_store!(
        (u16|i16) where wide: 32; mask: 1u16 << 15
    );

    unsafe fn write_packed(dst: &mut Vec<Self>, src: &[Self::Wide]) {
        for chunk in src.array_chunks::<2>() {
            let bitmask = chunk
                .map(|wide| Self::packed_mask(wide, Self::Wide::splat(Self::MASK)).to_bitmask());
            let cast: [u16; 4] = unsafe { mem::transmute(bitmask) };
            dst.extend_from_slice(&cast);
        }
    }
}

unsafe impl Store for u32 {
    impl_store!(
        (u32|i32) where wide: 16; mask: 1u32 << 31
    );

    unsafe fn write_packed(dst: &mut Vec<Self>, src: &[Self::Wide]) {
        for chunk in src.array_chunks::<2>() {
            let bitmask = chunk
                .map(|wide| Self::packed_mask(wide, Self::Wide::splat(Self::MASK)).to_bitmask());
            let bytes: [u8; 4] = unsafe { mem::transmute(bitmask) };
            dst.push(u32::from_le_bytes(bytes))
        }
    }
}

unsafe impl Store for u64 {
    impl_store!(
        (u64|i64) where wide: 8; mask: 1u64 << 63
    );

    unsafe fn write_packed(dst: &mut Vec<Self>, src: &[Self::Wide]) {
        for chunk in src.array_chunks::<8>() {
            let bitmask = chunk
                .map(|wide| Self::packed_mask(wide, Self::Wide::splat(Self::MASK)).to_bitmask());
            dst.push(u64::from_le_bytes(bitmask))
        }
    }
}

fn again_aligned(n: usize, align: usize) -> usize {
    (n + align - 1) / align
}

fn nearest_aligned(n: usize, align: usize) -> usize {
    n / align * align
}

/// Metadata for reverse join into bytes
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Meta {
    /// Offset to the align of `T::Wide`
    pub(crate) aligned_up: usize,
    /// Original len in bytes
    pub(crate) len: usize,
}

impl Meta {
    pub fn aligned_up(&self) -> usize {
        self.aligned_up
    }

    pub fn len(&self) -> usize {
        self.aligned_up
    }
}

pub fn split_into<T: Store>(slice: &[u8]) -> (Vec<T>, Meta) {
    // FIXME(temp-fix): few people will need to handle empty [u8]
    if slice.is_empty() {
        return (Vec::new(), Meta { aligned_up: 0, len: 0 });
    }

    let (head, tail) = slice.split_at(nearest_aligned(slice.len(), mem::size_of::<T>()));
    let bytes_len = slice.len();

    let len = head.len() / mem::size_of::<T>();
    // do not use `again_aligned` because `BitVec` may be reallocated
    let bits_len = len / bits_of::<T>() + 2;
    // ...
    let mut vec = Vec::<T>::with_capacity(len + bits_len + 1).tap_mut(|vec| {
        // SAFETY: uninit place of vector is a valid byte slice
        // `T: Store` must be friendly for initialization from bytes (or one of: u8..u64)
        unsafe {
            slice::from_raw_parts_mut(vec.as_mut_ptr() as *mut u8, head.len())
                .copy_from_slice(head);
            vec.set_len(len);
        }
    });
    let anchor_cap = vec.capacity();

    if !tail.is_empty() {
        vec.push(T::fill_up(tail));
    }

    // SAFETY: we leak `Vec`as owner but can guarantee that vector will not be reallocated
    // `slice` is an aligned as `T::Wide` part of vector place
    // `head` + `slice` + `tail` == `vector place`
    let (head, slice, tail) =
        unsafe { slice::from_raw_parts_mut(vec.as_mut_ptr(), vec.len()).align_to() };

    let (head, head_up) = T::fill_bits_up(head);
    T::write_pack(&mut vec, head, head_up);

    // SAFETY: we should believe that `Store::write_packed` will not overflow pre-alloc capacity
    // we check it in the next `debug_assert` with `anchor_cap`
    unsafe {
        T::write_packed(&mut vec, slice);
    }

    (
        BitVec::<_, Lsb0>::from_vec(vec)
            .tap_mut(|vec| {
                for _ in tail {
                    vec.push(false)
                }
            })
            .into_vec()
            // TODO(possible): maybe kept `.tap` instead of `.tap_dbg` or `debug_assert`
            .tap(|vec| assert_eq!(anchor_cap, vec.capacity())),
        Meta { aligned_up: head_up, len: bytes_len },
    )
}

pub fn join_into<T: Store>(slice: &[T], Meta { len, .. }: Meta) -> Vec<u8> {
    unsafe { Vec::from(slice::from_raw_parts(slice.as_ptr() as *const u8, len)) }
}

#[test]
fn prop() {
    fn inner<T: Store>(bytes: &[u8]) {
        let (vec, meta) = split_into::<T>(bytes);
        let new = join_into(&vec, meta);

        assert_eq!(new, bytes);
    }

    use proptest::{
        prelude::{any, prop::collection},
        test_runner::{Config, TestRunner},
    };

    TestRunner::new(Config { source_file: Some("src/lib.rs"), ..Config::default() })
        .run(&collection::vec(any::<u8>(), 0..512), |bytes| {
            let (chunks, len) = old::split_chunks(&bytes);
            assert_eq!(bytes, old::join_chunks(&chunks, len));

            inner::<u8>(&bytes);
            inner::<u16>(&bytes);
            inner::<u32>(&bytes);
            inner::<u64>(&bytes);
            Ok(())
        })
        .expect("tests are failed");
}

#[test]
#[cfg_attr(not(miri), ignore)]
fn miri() {
    let bytes = [0u8; 100];
    let _ = split_into::<u8>(&bytes);
    let _ = split_into::<u16>(&bytes);
    let _ = split_into::<u32>(&bytes);
    let _ = split_into::<u64>(&bytes);
}
