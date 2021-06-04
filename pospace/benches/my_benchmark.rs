use criterion::{Criterion, black_box, criterion_group, criterion_main};
use spaceframe_pospace::{bits::to_bits, fx_calculator::calculate_blake_hash};

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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
