//! Benchmark de la simulation — mesure les ticks/s a differentes populations.
//! Lancer avec : cargo bench --bench simulation

use std::time::Instant;

use garden_core::application::config::SimConfig;
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::world::{World, DEFAULT_GRID_SIZE};
use garden_core::infra::noise::generate_island;
use garden_core::infra::rng::SeededRng;

fn bench_ticks(population: usize, num_ticks: u32) -> f64 {
    let config = SimConfig {
        initial_population: population,
        ..SimConfig::default()
    };

    let mut rng = SeededRng::new(42);
    let mut world = World::new(DEFAULT_GRID_SIZE);
    let island = generate_island(&mut world, 42, 0.2);
    let mut state = SimState::with_terrain(world, island, config, &mut rng);

    // Warmup : laisser la simulation se stabiliser
    for _ in 0..100 {
        run_tick(&mut state, &mut rng);
    }

    let alive = state.plants.iter().filter(|p| !p.is_dead()).count();

    // Bench
    let start = Instant::now();
    for _ in 0..num_ticks {
        run_tick(&mut state, &mut rng);
    }
    let elapsed = start.elapsed();

    let ticks_per_sec = num_ticks as f64 / elapsed.as_secs_f64();

    println!(
        "  pop_init={:<4} pop_reelle={:<4} | {} ticks en {:.2}s | {:.0} ticks/s",
        population,
        alive,
        num_ticks,
        elapsed.as_secs_f64(),
        ticks_per_sec
    );

    ticks_per_sec
}

fn main() {
    println!("=== Neural Garden — Benchmark Simulation ===\n");

    let configs = vec![
        (30, 500),  // population petite, 500 ticks
        (50, 500),  // population moyenne
        (100, 300), // population grande
        (200, 200), // population tres grande
    ];

    for (pop, ticks) in &configs {
        bench_ticks(*pop, *ticks);
    }

    println!("\n=== Fin du benchmark ===");
}
