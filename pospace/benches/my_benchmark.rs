use bitvec::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use spaceframe_pospace::bits::BitsWrapper;
use spaceframe_pospace::constants::PARAM_EXT;
use spaceframe_pospace::core::PoSpace;
use spaceframe_pospace::f1_calculator::F1Calculator;
use spaceframe_pospace::proofs::Prover;
use spaceframe_pospace::{
    bits::{from_bits, to_bits},
    fx_calculator::FxCalculator,
};
use tempdir::TempDir;

const fn get_challenge(k: usize) -> [u8; 32] {
    match k {
        17 => [
            95, 23, 106, 107, 81, 99, 43, 112, 157, 49, 246, 48, 199, 114, 163, 190, 160, 165, 251,
            13, 92, 80, 240, 210, 241, 247, 44, 74, 94, 126, 245, 226,
        ],
        19 => [
            140, 31, 177, 106, 121, 35, 250, 68, 109, 103, 251, 149, 126, 201, 224, 230, 37, 74,
            247, 24, 146, 131, 28, 74, 17, 105, 126, 93, 105, 34, 222, 152,
        ],
        20 => [
            8, 69, 62, 233, 63, 175, 0, 92, 104, 211, 47, 131, 61, 52, 7, 0, 19, 150, 63, 103, 88,
            212, 133, 181, 140, 197, 12, 27, 33, 249, 33, 196,
        ],
        21 => [
            154, 156, 38, 140, 105, 6, 177, 113, 168, 152, 154, 83, 173, 244, 200, 201, 218, 49,
            102, 110, 98, 200, 99, 103, 187, 151, 182, 107, 149, 19, 244, 32,
        ],
        22 => [
            95, 173, 51, 119, 88, 121, 172, 0, 127, 130, 7, 43, 153, 16, 149, 83, 31, 188, 77, 86,
            226, 139, 33, 67, 232, 168, 112, 191, 24, 57, 130, 138,
        ],
        _ => [0u8; 32],
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut calculate_fn_group = c.benchmark_group("calculate_fn");
    calculate_fn_group.bench_function("calculate_f1", |b| {
        let fx = F1Calculator::new(12, *b"aaaabbbbccccddddaaaabbbbccccdddd");
        b.iter(|| fx.calculate_f1(black_box(&BitsWrapper::from(0xabcd, 12))))
    });

    calculate_fn_group.bench_function("calculate_fn_table2", |b| {
        let fx = FxCalculator::new(12, 2);
        let y1 = to_bits(0xabcbef, 12 + PARAM_EXT);
        let left = to_bits(0xabcd, 12);
        let right = to_bits(0xefab, 12);
        b.iter(|| fx.calculate_fn(black_box(&y1), black_box(&left), black_box(&right)))
    });

    calculate_fn_group.bench_function("calculate_fn_table3", |b| {
        let fx = FxCalculator::new(12, 3);
        let y1 = to_bits(0xabcbef, 12 + PARAM_EXT);
        let left = to_bits(0xabcd, 12);
        let right = to_bits(0xefab, 12);
        b.iter(|| fx.calculate_fn(black_box(&y1), black_box(&left), black_box(&right)))
    });

    calculate_fn_group.bench_function("calculate_fn_table4", |b| {
        let fx = FxCalculator::new(12, 4);
        let y1 = to_bits(0xabcbef, 12 + PARAM_EXT);
        let left = to_bits(0xabcd, 12);
        let right = to_bits(0xefab, 12);
        b.iter(|| fx.calculate_fn(black_box(&y1), black_box(&left), black_box(&right)))
    });
    calculate_fn_group.finish();

    let mut bits_group = c.benchmark_group("bits");
    bits_group.bench_function("to_bits", |b| {
        b.iter(|| {
            to_bits(black_box(0xabcd), black_box(16));
        })
    });
    bits_group.bench_function("from_bits", |b| {
        let input = bitvec![Lsb0, u8; 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1];
        b.iter(|| {
            from_bits(black_box(&input));
        })
    });
    bits_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
