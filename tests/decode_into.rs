//! `FastPForCodec::decode_into` must decode exactly what `AnyLenCodec::decode` does.

#![cfg(feature = "rust")]
#![allow(missing_docs, clippy::unwrap_used)]

use fastpfor::{
    AnyLenCodec, FastPForError, FastPForInterleaved32x128, FastPForInterleaved32x256,
    FastPForInterleaved64x128, FastPForInterleaved64x256, FastPForSequential32x128,
    FastPForSequential32x256, FastPForSequential64x128, FastPForSequential64x256,
};
use rand::rngs::StdRng;
use rand::{RngExt as _, SeedableRng as _};

/// Shared checks, written once for every codec type through a macro.
macro_rules! check_codec {
    ($codec:ty, $t:ty, $data:expr) => {{
        let data: Vec<$t> = $data;
        let mut codec = <$codec>::default();
        let mut encoded = Vec::new();
        codec.encode(&data, &mut encoded).unwrap();

        let mut expected = Vec::new();
        codec.decode(&encoded, &mut expected, None).unwrap();
        assert_eq!(expected, data);

        // Exactly the right size, and larger, reused across calls.
        let mut out = vec![<$t>::MAX; data.len()];
        assert_eq!(codec.decode_into(&encoded, &mut out).unwrap(), data.len());
        assert_eq!(out, data);
        let mut big = vec![<$t>::MAX; data.len() + 300];
        for _ in 0..2 {
            assert_eq!(codec.decode_into(&encoded, &mut big).unwrap(), data.len());
            assert_eq!(&big[..data.len()], &data[..]);
        }
        // One value short is an error, never a panic or a partial result.
        if !data.is_empty() {
            let mut small = vec![0; data.len() - 1];
            let err = codec.decode_into(&encoded, &mut small).unwrap_err();
            assert!(
                matches!(err, FastPForError::OutputBufferTooSmall),
                "{err:?}"
            );
        }
        // Corrupted streams: `decode_into` fails exactly when `decode` does, and agrees otherwise.
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..50 {
            // Small inputs reach every part of the stream; large ones only slow the test down.
            if encoded.is_empty() || data.len() > 1000 {
                break;
            }
            let mut bad = encoded.clone();
            let i = rng.random_range(0..bad.len());
            bad[i] ^= 1 << rng.random_range(0..32);
            let mut via_vec = Vec::new();
            let vec_result = codec.decode(&bad, &mut via_vec, None);
            let mut via_slice = vec![0; via_vec.len().max(data.len()) + 300];
            let slice_result = codec.decode_into(&bad, &mut via_slice);
            assert_eq!(
                vec_result.is_ok(),
                slice_result.is_ok(),
                "{vec_result:?} {slice_result:?}"
            );
            if let Ok(n) = slice_result {
                assert_eq!(&via_slice[..n], &via_vec[..]);
            }
        }
    }};
}

fn sizes() -> [usize; 8] {
    [0, 1, 127, 128, 300, 512, 65536 + 77, 2 * 65536 + 300]
}

fn values32(n: usize, seed: u64) -> Vec<u32> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..n)
        .map(|i| match i % 7 {
            0 => rng.random(),
            1..=3 => rng.random_range(0..16),
            _ => rng.random_range(0..5000),
        })
        .collect()
}

fn values64(n: usize, seed: u64) -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..n)
        .map(|i| match i % 7 {
            0 => rng.random(),
            1 => rng.random_range(0..1 << 40),
            _ => rng.random_range(0..5000),
        })
        .collect()
}

#[test]
fn decode_into_matches_decode_u32() {
    for (seed, n) in sizes().into_iter().enumerate() {
        let seed = seed as u64;
        check_codec!(FastPForSequential32x128, u32, values32(n, seed));
        check_codec!(FastPForSequential32x256, u32, values32(n, seed));
        check_codec!(FastPForInterleaved32x128, u32, values32(n, seed));
        check_codec!(FastPForInterleaved32x256, u32, values32(n, seed));
    }
}

#[test]
fn decode_into_matches_decode_u64() {
    for (seed, n) in sizes().into_iter().enumerate() {
        let seed = seed as u64;
        check_codec!(FastPForSequential64x128, u64, values64(n, seed));
        check_codec!(FastPForSequential64x256, u64, values64(n, seed));
        check_codec!(FastPForInterleaved64x128, u64, values64(n, seed));
        check_codec!(FastPForInterleaved64x256, u64, values64(n, seed));
    }
}

#[test]
fn decode_into_empty_input() {
    let mut out = [0u32; 4];
    assert_eq!(
        FastPForSequential32x128::default()
            .decode_into(&[], &mut out)
            .unwrap(),
        0
    );
}
