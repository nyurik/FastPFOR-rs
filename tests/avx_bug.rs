//! Minimal reproduction of the AVX bug that causes SIGILL on AMD Server CPUs.
//!
//! The bug was first observed in:
//! - <https://github.com/maplibre/maplibre-tile-spec/issues/1016>
//! - <https://github.com/maplibre/maplibre-tile-spec/actions/runs/22491110985/job/65153285092>
//!
//! On AMD EPYC 7763 (AVX2, no AVX-512), encoding 256 identical large values with
//! the SIMD-based `FastPFor` codecs crashes with `signal: 4, SIGILL: illegal instruction`.

#![cfg(feature = "cpp")]

use fastpfor::cpp::{Codec32 as _, FastPFor256Codec, SimdFastPFor256Codec};

/// Reproduces the original maplibre bug: `test_fastpfor_roundtrip::case_2_large`.
///
/// Encoding a block of 256 identical large values (filling a full SIMD block) triggers
/// an illegal instruction on AMD Server CPUs that have AVX2 but not AVX-512.
#[test]
fn test_fastpfor256_large_repeated_values() {
    let input = vec![1_000_000u32; 256];
    let mut compressed = vec![0u32; input.len() + 1024];

    let codec = FastPFor256Codec::new();
    let out = codec.encode32(&input, &mut compressed).unwrap();

    let mut decoded = vec![0u32; input.len()];
    let result = codec.decode32(out, &mut decoded).unwrap();

    assert_eq!(result, &input[..]);
}

/// Same as above but with the explicit SIMD variant of the codec.
#[test]
fn test_simd_fastpfor256_large_repeated_values() {
    let input = vec![1_000_000u32; 256];
    let mut compressed = vec![0u32; input.len() + 1024];

    let codec = SimdFastPFor256Codec::new();
    let out = codec.encode32(&input, &mut compressed).unwrap();

    let mut decoded = vec![0u32; input.len()];
    let result = codec.decode32(out, &mut decoded).unwrap();

    assert_eq!(result, &input[..]);
}
