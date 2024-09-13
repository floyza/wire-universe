use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::Path;
use wire_universe_server::world::World;

fn simulate_computer(cycles: u64) {
    let mut world = World::from_wi(Path::new("../primes.wi")).unwrap();
    for _ in 0..cycles {
        world.step();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_with_input(BenchmarkId::new("computer_sim", 5), &5, |b, &s| {
        b.iter(|| simulate_computer(s))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
