use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rand_pcg::Pcg64;
use rand_indices::rand_indices::RngExt;
use rand::SeedableRng;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tuples");
    let mut rng = Pcg64::seed_from_u64(1);
    group.bench_function(BenchmarkId::new("Uniform", 0),
                         |b| b.iter(|| rng.random_distinct_index_tuple_ordered_except_good(5, (0, 2))));
    group.bench_function(BenchmarkId::new("Non-Uniform", 0),
                         |b| b.iter(|| rng.random_distinct_index_tuple_ordered_except_fast(5, (0, 2))));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);