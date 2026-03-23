// Runner multi-thread : la simulation tourne dans un thread separe
// et communique avec le thread UI via un channel mpsc.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

use garden_core::application::highlights::{Highlight, HighlightType};
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::plant::{PlantState, Pos};
use garden_core::infra::persistence::{auto_save, get_auto_save_slot, should_auto_save};
use garden_core::infra::replay::{ReplayConfig, ReplayRecorder};
use garden_core::infra::rng::SeededRng;

use crate::snapshot::SimSnapshot;

/// Controles de la simulation partages entre threads.
#[derive(Clone)]
pub struct SimControls {
    pub paused: Arc<AtomicBool>,
    pub quit: Arc<AtomicBool>,
    pub save_requested: Arc<AtomicBool>,
    /// Indique si la vue ile est active (calque a scanner)
    pub island_active: Arc<AtomicBool>,
    /// Numero du calque ile demande (0=plantes, 1=C, 2=N, 3=H, 4=racines, 5=canopee, 6=footprint)
    pub island_layer: Arc<AtomicU8>,
}

impl SimControls {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            quit: Arc::new(AtomicBool::new(false)),
            save_requested: Arc::new(AtomicBool::new(false)),
            island_active: Arc::new(AtomicBool::new(false)),
            island_layer: Arc::new(AtomicU8::new(0)),
        }
    }
}

/// Lance la simulation dans un thread separe.
/// Retourne le JoinHandle du thread.
pub fn spawn_simulation(
    mut state: SimState,
    mut rng: SeededRng,
    controls: SimControls,
    snapshot_tx: mpsc::Sender<SimSnapshot>,
) -> thread::JoinHandle<()> {
    let paused = controls.paused.clone();
    let quit = controls.quit.clone();
    let save_requested = controls.save_requested.clone();
    let island_active = controls.island_active.clone();
    let island_layer = controls.island_layer.clone();

    thread::spawn(move || {
        let mut recorder = ReplayRecorder::new(ReplayConfig::default());
        let mut last_snapshot = Instant::now();
        let mut tick_count_at_last_snapshot = state.tick_count;

        // Cache du calque ile (scan lazy toutes les 500ms)
        let mut island_cache: Vec<f32> = Vec::new();
        let mut island_cache_layer: u8 = 0;
        let mut last_island_scan = Instant::now();

        loop {
            if quit.load(Ordering::Relaxed) {
                // Sauvegarde finale avant de quitter
                let _ = auto_save(&state, Path::new("saves"), 1);
                break;
            }

            if paused.load(Ordering::Relaxed) {
                // Scan calque ile toutes les 500ms si actif (meme en pause)
                if island_active.load(Ordering::Relaxed)
                    && last_island_scan.elapsed().as_millis() > 500
                {
                    let requested = island_layer.load(Ordering::Relaxed);
                    island_cache = scan_island_layer(&state, requested);
                    island_cache_layer = requested;
                    last_island_scan = Instant::now();
                }

                // Envoyer un snapshot meme en pause (pour afficher PAUSE)
                if last_snapshot.elapsed().as_millis() > 100 {
                    let snapshot =
                        build_snapshot(&state, 0.0, true, &island_cache, island_cache_layer);
                    let _ = snapshot_tx.send(snapshot);
                    last_snapshot = Instant::now();
                }
                thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }

            // Sauvegarde manuelle si demandee
            if save_requested.swap(false, Ordering::Relaxed) {
                let _ = auto_save(&state, Path::new("saves"), 1);
            }

            // Executer un tick
            let events = run_tick(&mut state, &mut rng);

            // Replay
            recorder.record_tick(state.tick_count, &events);
            let highlights: Vec<_> = state.metrics.recent_highlights.clone();
            recorder.process_highlights(state.tick_count, &highlights);

            // Auto-save toutes les 1000 ticks
            if should_auto_save(state.tick_count, 1000) {
                let slot = get_auto_save_slot(state.tick_count, 3, 1000);
                let _ = auto_save(&state, Path::new("saves"), slot);
            }

            // Montage replay toutes les 5000 ticks
            if state.tick_count.is_multiple_of(5000) && state.tick_count > 0 {
                recorder.finalize_clips(state.tick_count);
                let _ = std::fs::create_dir_all("replays");
                let _ = recorder.save_montage(
                    Path::new(&format!("replays/montage_{:06}.json", state.tick_count)),
                    state.tick_count,
                );
            }

            // Scan calque ile toutes les 500ms si actif
            if island_active.load(Ordering::Relaxed) && last_island_scan.elapsed().as_millis() > 500
            {
                let requested = island_layer.load(Ordering::Relaxed);
                island_cache = scan_island_layer(&state, requested);
                island_cache_layer = requested;
                last_island_scan = Instant::now();
            }

            // Envoyer un snapshot toutes les ~33ms (30fps)
            let elapsed = last_snapshot.elapsed();
            if elapsed.as_millis() > 33 {
                let ticks_done = state.tick_count - tick_count_at_last_snapshot;
                let tps = ticks_done as f64 / elapsed.as_secs_f64();
                let snapshot =
                    build_snapshot(&state, tps, false, &island_cache, island_cache_layer);
                let _ = snapshot_tx.send(snapshot);
                last_snapshot = Instant::now();
                tick_count_at_last_snapshot = state.tick_count;
            }
        }
    })
}

/// Construit un SimSnapshot depuis l'etat actuel.
fn build_snapshot(
    state: &SimState,
    ticks_per_second: f64,
    paused: bool,
    island_cache: &[f32],
    island_cache_layer: u8,
) -> SimSnapshot {
    // Mini-carte : reduire la grille (chaque pixel = 4x4 cellules)
    let grid_size = state.world.size();
    let map_size = (grid_size / 4) as usize;
    let mut minimap = vec![vec![0u8; map_size]; map_size];

    for (my, row) in minimap.iter_mut().enumerate() {
        for (mx, cell) in row.iter_mut().enumerate() {
            let pos = Pos {
                x: (mx * 4) as u16,
                y: (my * 4) as u16,
            };
            if state.island.is_land(&pos) {
                *cell = 1; // terre vide
            }
            // 0 = mer (defaut)
        }
    }

    // Marquer les plantes sur la minimap
    for plant in &state.plants {
        if plant.is_dead() {
            continue;
        }
        for cell_pos in plant.footprint() {
            let mx = (cell_pos.x / 4) as usize;
            let my = (cell_pos.y / 4) as usize;
            if mx < map_size && my < map_size {
                minimap[my][mx] = if plant.state() == PlantState::Mature {
                    3 // plante mature
                } else {
                    2 // plante
                };
            }
        }
    }

    // Calques de l'ile
    let mut layer_carbon = vec![vec![0u8; map_size]; map_size];
    let mut layer_nitrogen = vec![vec![0u8; map_size]; map_size];
    let mut layer_humidity = vec![vec![0u8; map_size]; map_size];

    for my in 0..map_size {
        for mx in 0..map_size {
            let pos = Pos {
                x: (mx * 4) as u16,
                y: (my * 4) as u16,
            };
            if let Some(cell) = state.world.get(&pos) {
                // Clamper les valeurs entre 0 et 1 avant conversion
                layer_carbon[my][mx] = (cell.carbon().clamp(0.0, 1.0) * 255.0) as u8;
                layer_nitrogen[my][mx] = (cell.nitrogen().clamp(0.0, 1.0) * 255.0) as u8;
                layer_humidity[my][mx] = (cell.humidity().clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
    }

    // Calques plantes : racines, canopee, footprint
    let mut layer_roots = vec![vec![0u8; map_size]; map_size];
    let mut layer_canopy = vec![vec![0u8; map_size]; map_size];
    let mut layer_footprint = vec![vec![0u8; map_size]; map_size];

    for plant in &state.plants {
        if plant.is_dead() {
            continue;
        }
        for pos in plant.roots() {
            let mx = (pos.x / 4) as usize;
            let my = (pos.y / 4) as usize;
            if mx < map_size && my < map_size {
                layer_roots[my][mx] = layer_roots[my][mx].saturating_add(50);
            }
        }
        for pos in plant.canopy() {
            let mx = (pos.x / 4) as usize;
            let my = (pos.y / 4) as usize;
            if mx < map_size && my < map_size {
                layer_canopy[my][mx] = layer_canopy[my][mx].saturating_add(50);
            }
        }
        for pos in plant.footprint() {
            let mx = (pos.x / 4) as usize;
            let my = (pos.y / 4) as usize;
            if mx < map_size && my < map_size {
                layer_footprint[my][mx] = 255; // occupe = max
            }
        }
    }

    // Formater les highlights en messages lisibles
    let recent_highlights: Vec<String> = state
        .metrics
        .recent_highlights
        .iter()
        .map(format_highlight)
        .collect();

    SimSnapshot {
        tick: state.tick_count,
        year: state.season_cycle.year(),
        season: state.season_cycle.current_season(),
        alive_count: state.metrics.alive_count,
        lineage_count: state.metrics.lineage_count,
        symbiosis_count: state.metrics.symbiosis_count,
        average_age: state.metrics.average_age,
        total_biomass: state.metrics.total_biomass,
        best_fitness: state.seed_bank.best_fitness(),
        worst_fitness: state.seed_bank.worst_fitness(),
        generation: state.generation_counter.current(),
        population_history: state.metrics.population_history.clone(),
        fitness_history: state.metrics.fitness_history.clone(),
        symbiosis_history: state.metrics.symbiosis_history.clone(),
        lineage_distribution: state.metrics.lineage_distribution.clone(),
        recent_highlights,
        paused,
        ticks_per_second,
        minimap,
        layer_carbon,
        layer_nitrogen,
        layer_humidity,
        layer_roots,
        layer_canopy,
        layer_footprint,
        active_layer: 0, // le calque actif est gere par main.rs
        island_layer_data: island_cache.to_vec(),
        island_layer_id: island_cache_layer,
        births_count: state.metrics.births_count,
        deaths_count: state.metrics.deaths_count,
        births_last_year: state.metrics.births_last_year,
        deaths_last_year: state.metrics.deaths_last_year,
        age_buckets: state.metrics.age_buckets,
        carbon_count: state.metrics.carbon_count,
        nitrogen_count: state.metrics.nitrogen_count,
        cooperators_count: state.metrics.cooperators_count,
        cooperators_ratio: state.metrics.cooperators_ratio,
        avg_soil_carbon: state.metrics.avg_soil_carbon,
        avg_soil_nitrogen: state.metrics.avg_soil_nitrogen,
        land_coverage: state.metrics.land_coverage,
        empty_land_cells: state.metrics.empty_land_cells,
        total_exchanges_2y: state.metrics.total_exchanges_2y,
        bank_compartments: state.metrics.bank_compartments,
        bank_total_genomes: state.metrics.bank_total_genomes,
        bank_best_fitness: state.metrics.bank_best_fitness,
        bank_worst_fitness: state.metrics.bank_worst_fitness,
        bank_spread: state.metrics.bank_spread,
        bank_top5: {
            let mut entries: Vec<(f32, u8, String, u16)> = state
                .seed_bank
                .entries()
                .iter()
                .map(|(genome, fitness)| {
                    (
                        *fitness,
                        genome.traits.hidden_size(),
                        format!("{:?}", genome.traits.exudate_type()),
                        genome.traits.max_size(),
                    )
                })
                .collect();
            entries.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            entries.truncate(5);
            entries
        },
    }
}

/// Scan un calque de l'ile a pleine resolution (world.size() x world.size()).
/// Retourne un Vec<f32> de taille world.size()*world.size().
/// layer: 0=rien, 1=carbone, 2=azote, 3=humidite, 4=racines, 5=canopee, 6=footprint
fn scan_island_layer(state: &SimState, layer: u8) -> Vec<f32> {
    let size = state.world.size() as usize;
    let mut data = vec![0.0f32; size * size];

    match layer {
        1 => {
            // Carbone
            for y in 0..state.world.size() {
                for x in 0..state.world.size() {
                    let pos = Pos { x, y };
                    if let Some(cell) = state.world.get(&pos) {
                        data[y as usize * size + x as usize] = cell.carbon();
                    }
                }
            }
        }
        2 => {
            // Azote
            for y in 0..state.world.size() {
                for x in 0..state.world.size() {
                    let pos = Pos { x, y };
                    if let Some(cell) = state.world.get(&pos) {
                        data[y as usize * size + x as usize] = cell.nitrogen();
                    }
                }
            }
        }
        3 => {
            // Humidite
            for y in 0..state.world.size() {
                for x in 0..state.world.size() {
                    let pos = Pos { x, y };
                    if let Some(cell) = state.world.get(&pos) {
                        data[y as usize * size + x as usize] = cell.humidity();
                    }
                }
            }
        }
        4 => {
            // Racines (densite)
            for plant in &state.plants {
                if plant.is_dead() {
                    continue;
                }
                for pos in plant.roots() {
                    let idx = pos.y as usize * size + pos.x as usize;
                    if idx < data.len() {
                        data[idx] += 1.0;
                    }
                }
            }
        }
        5 => {
            // Canopee (densite)
            for plant in &state.plants {
                if plant.is_dead() {
                    continue;
                }
                for pos in plant.canopy() {
                    let idx = pos.y as usize * size + pos.x as usize;
                    if idx < data.len() {
                        data[idx] += 1.0;
                    }
                }
            }
        }
        6 => {
            // Footprint (occupation)
            for plant in &state.plants {
                if plant.is_dead() {
                    continue;
                }
                for pos in plant.footprint() {
                    let idx = pos.y as usize * size + pos.x as usize;
                    if idx < data.len() {
                        data[idx] = 1.0;
                    }
                }
            }
        }
        _ => {} // 0 = vue plantes, pas de scan
    }

    data
}

/// Formate un highlight en message lisible pour le panneau alertes.
fn format_highlight(h: &Highlight) -> String {
    match &h.highlight_type {
        HighlightType::FirstSymbiosis => {
            format!("[+] Tick {} — Première symbiose !", h.tick)
        }
        HighlightType::FitnessRecord { fitness } => {
            format!("[!] Tick {} — Record fitness : {:.1}", h.tick, fitness)
        }
        HighlightType::MajorInvasion { cells_taken, .. } => {
            format!(
                "[!] Tick {} — Invasion majeure ({} cellules)",
                h.tick, cells_taken
            )
        }
        HighlightType::MassDeath { deaths, .. } => {
            format!("[x] Tick {} — Mort de masse ({} morts)", h.tick, deaths)
        }
        HighlightType::PopulationBoom { population } => {
            format!(
                "[+] Tick {} — Boom démographique (pop: {})",
                h.tick, population
            )
        }
        HighlightType::SeasonChange { season } => {
            format!("[~] Tick {} — {:?}", h.tick, season)
        }
        HighlightType::LineageExtinction { lineage_id } => {
            format!("[x] Tick {} — Lignée {} éteinte", h.tick, lineage_id)
        }
        HighlightType::NewLineage { lineage_id, .. } => {
            format!("[+] Tick {} — Nouvelle lignée {}", h.tick, lineage_id)
        }
    }
}
