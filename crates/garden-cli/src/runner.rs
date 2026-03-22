// Runner multi-thread : la simulation tourne dans un thread separe
// et communique avec le thread UI via un channel mpsc.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

use garden_core::application::highlights::{Highlight, HighlightType};
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::plant::{PlantState, Pos};
use garden_core::domain::world::GRID_SIZE;
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
}

impl SimControls {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            quit: Arc::new(AtomicBool::new(false)),
            save_requested: Arc::new(AtomicBool::new(false)),
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

    thread::spawn(move || {
        let mut recorder = ReplayRecorder::new(ReplayConfig::default());
        let mut last_snapshot = Instant::now();
        let mut tick_count_at_last_snapshot = state.tick_count;

        loop {
            if quit.load(Ordering::Relaxed) {
                // Sauvegarde finale avant de quitter
                let _ = auto_save(&state, Path::new("saves"), 1);
                break;
            }

            if paused.load(Ordering::Relaxed) {
                // Envoyer un snapshot meme en pause (pour afficher PAUSE)
                if last_snapshot.elapsed().as_millis() > 100 {
                    let snapshot = build_snapshot(&state, 0.0, true);
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

            // Envoyer un snapshot toutes les ~33ms (30fps)
            let elapsed = last_snapshot.elapsed();
            if elapsed.as_millis() > 33 {
                let ticks_done = state.tick_count - tick_count_at_last_snapshot;
                let tps = ticks_done as f64 / elapsed.as_secs_f64();
                let snapshot = build_snapshot(&state, tps, false);
                let _ = snapshot_tx.send(snapshot);
                last_snapshot = Instant::now();
                tick_count_at_last_snapshot = state.tick_count;
            }
        }
    })
}

/// Construit un SimSnapshot depuis l'etat actuel.
fn build_snapshot(state: &SimState, ticks_per_second: f64, paused: bool) -> SimSnapshot {
    // Mini-carte : reduire la grille (chaque pixel = 4x4 cellules)
    let map_size = (GRID_SIZE / 4) as usize;
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
    }
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
