use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::Path;
use wire_universe_server::world::World;

fn simulate_computer(cycles: u64, mut world: World) {
    for _ in 0..cycles {
        world.step();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let world = World::from_wi(Path::new("../primes.wi")).unwrap();
    c.bench_with_input(BenchmarkId::new("computer_sim", 1000), &1000, |b, &s| {
        b.iter(|| simulate_computer(s, world.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
