use bitvec::{bitvec, order::Lsb0};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{RngCore, rngs::OsRng};
use spaceframe_pospace::{
    bits::{from_bits, to_bits, BitsWrapper},
    core::PoSpace,
    fx_calculator::{calculate_blake_hash, FXCalculator},
};

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
        let in_c = to_bits(0xef, 10);
        b.iter(|| {
            fx_calculator.calculate_fn(black_box(&[&in_a, &in_b]), black_box( &in_c));
        })
    });

    c.bench_function("calculate_f3", |b| {
        let fx_calculator = FXCalculator::new(16);
        let in_a = to_bits(0xabcd, 16);
        let in_b = to_bits(0xcdef, 16);
        let in_c = to_bits(0x1234, 16);
        let in_d = to_bits(0x5678, 16);
        let in_e = to_bits(0xef, 10);
        b.iter(|| {
            fx_calculator.calculate_fn(black_box(&[&in_a, &in_b, &in_c, &in_d]),  &in_e);
        })
    });

    c.bench_function("matching", |b| {
        let pospace = PoSpace::new(14, b"some key");
        let l = BitsWrapper::new(bitvec![Lsb0, u8; 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1]);
        let r = BitsWrapper::new(bitvec![Lsb0, u8; 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0]);
        b.iter(|| {
            pospace.matching(&l, &r);
        })
    });

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
}

fn phase1_benchmark(c: &mut Criterion) {
    let mut plot_seed = [0u8; 32];
    OsRng.fill_bytes(&mut plot_seed);
    let pos = PoSpace::new(10, &plot_seed);

    c.bench_function("run_phase1", |b| {
        b.iter(|| pos.run_phase_1());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_group!{
    name = phase1;
    config = Criterion::default().sample_size(50).significance_level(0.1).noise_threshold(0.05);
    targets = phase1_benchmark
}
criterion_main!(benches, phase1);
