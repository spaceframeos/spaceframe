use bitvec::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spaceframe_pospace::bits::BitsWrapper;
use spaceframe_pospace::constants::PARAM_EXT;
use spaceframe_pospace::f1_calculator::F1Calculator;
use spaceframe_pospace::{
    bits::{from_bits, to_bits},
    fx_calculator::FxCalculator,
};

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
