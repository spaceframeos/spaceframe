use bitvec::{bitvec, order::Lsb0};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spaceframe_pospace::{bits::{from_bits, to_bits}, fx_calculator::{calculate_blake_hash, FXCalculator}};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("blake3", |b| {
        b.iter(|| {
            calculate_blake_hash(
                black_box(&to_bits(0xab, 10)),
                black_box(&to_bits(0xabcd, 16)),
                black_box(&to_bits(0xef34, 16)),
            )
        })
    });

    c.bench_function("calculate_f2", |b| {
        let fx_calculator = FXCalculator::new(10);
        let in_a = to_bits(0xab, 10);
        let in_b = to_bits(0xcd, 10);
        b.iter(|| {
            fx_calculator.calculate_fn(black_box(&[&in_a, &in_b]));
        })
    });

    c.bench_function("calculate_f3", |b| {
        let fx_calculator = FXCalculator::new(16);
        let in_a = to_bits(0xabcd, 16);
        let in_b = to_bits(0xcdef, 16);
        let in_c = to_bits(0x1234, 16);
        let in_d = to_bits(0x5678, 16);
        b.iter(|| {
            fx_calculator.calculate_fn(black_box(&[&in_a, &in_b, &in_c, &in_d]));
        })
    });

    c.bench_function("to_bits", |b| {
        b.iter(|| {
            to_bits(black_box(0xabcd), black_box(16));
        })
    });

    c.bench_function("from_bits", |b| {
        b.iter(|| {
            from_bits(black_box(&bitvec![Lsb0, u8; 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1]));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
