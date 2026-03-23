// CLI pour Neural Garden — lance la simulation ou gere la configuration.

mod live;
mod nursery_runner;
mod nursery_snapshot;
mod runner;
mod server;
mod snapshot;
mod tui;
mod ui;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::world::{World, DEFAULT_GRID_SIZE};
use garden_core::infra::config::{generate_default_toml, load_config};
use garden_core::infra::noise::generate_island;
use garden_core::infra::persistence::{
    auto_save, get_auto_save_slot, load_state, should_auto_save,
};
use garden_core::infra::replay::{ReplayConfig, ReplayRecorder};
use garden_core::infra::rng::SeededRng;

use crate::nursery_runner::{spawn_nursery, NurseryControls};
use crate::nursery_snapshot::{NurserySnapshot, NurseryViewMode};
use crate::runner::{spawn_simulation, SimControls};
use crate::snapshot::SimSnapshot;
use crate::tui::Tui;
use crate::ui::AppMode;

#[derive(Parser)]
#[command(
    name = "garden",
    about = "Neural Garden — simulateur de neuroevolution"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lancer la simulation
    Run {
        /// Chemin vers le fichier de configuration TOML
        #[arg(short, long, default_value = "garden.toml")]
        config: String,

        /// Reprendre depuis une sauvegarde
        #[arg(short, long)]
        resume: Option<String>,

        /// Desactiver le TUI (affichage texte simple)
        #[arg(long)]
        no_tui: bool,
    },
    /// Gerer la configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Servir le viewer web avec un fichier de montage
    Replay {
        /// Chemin vers le fichier de montage JSON
        montage: String,
        /// Port HTTP
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Lancer la simulation en mode live avec WebSocket
    Live {
        /// Chemin vers le fichier de configuration TOML
        #[arg(short, long, default_value = "garden.toml")]
        config: String,
        /// Port HTTP pour le viewer
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Port WebSocket
        #[arg(long, default_value = "8080")]
        ws_port: u16,
    },
    /// Pepiniere — pre-entrainement genetique des graines
    Nursery {
        /// Chemin vers le fichier de config des environnements
        #[arg(short, long, default_value = "configs/nursery/environments.yaml")]
        config: String,

        /// Nombre de generations
        #[arg(long, default_value_t = 50)]
        generations: u32,

        /// Taille de la population par generation
        #[arg(long, default_value_t = 50)]
        population: usize,

        /// Seed pour la reproductibilite
        #[arg(long, default_value_t = 42)]
        seed: u64,

        /// Mode verbose — affiche les stats detaillees du champion
        #[arg(long)]
        verbose: bool,

        /// Banque de graines pour reprendre un entrainement
        #[arg(long)]
        bank: Option<String>,

        /// Mode sans interface — logs texte
        #[arg(long)]
        no_tui: bool,

        /// Action optionnelle (commit)
        #[command(subcommand)]
        action: Option<NurseryAction>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Generer un fichier de configuration par defaut
    Init,
}

#[derive(Subcommand)]
enum NurseryAction {
    /// Figer les meilleurs genomes dans une banque versionnable
    Commit {
        /// Fichier de sortie (ex: seeds/v1.json)
        #[arg(long)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run {
            config,
            resume,
            no_tui,
        } => cmd_run(&config, resume.as_deref(), no_tui),
        Commands::Config { action } => match action {
            ConfigAction::Init => cmd_config_init(),
        },
        Commands::Replay { montage, port } => cmd_replay(&montage, port),
        Commands::Live {
            config,
            port,
            ws_port,
        } => cmd_live(&config, port, ws_port),
        Commands::Nursery {
            config,
            generations,
            population,
            seed,
            verbose,
            bank,
            no_tui,
            action,
        } => {
            if no_tui {
                cmd_nursery_headless(
                    &config,
                    generations,
                    population,
                    seed,
                    verbose,
                    bank.as_deref(),
                    action,
                )
            } else {
                cmd_nursery_tui(
                    &config,
                    generations,
                    population,
                    seed,
                    bank.as_deref(),
                    action,
                )
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Erreur: {e}");
        std::process::exit(1);
    }
}

/// Lance la simulation (nouvelle ou reprise).
fn cmd_run(config_path: &str, resume: Option<&str>, no_tui: bool) -> Result<(), String> {
    // Creer les dossiers necessaires
    fs::create_dir_all("saves")
        .map_err(|e| format!("impossible de creer le dossier saves/: {e}"))?;
    fs::create_dir_all("replays")
        .map_err(|e| format!("impossible de creer le dossier replays/: {e}"))?;

    let (state, rng) = if let Some(path) = resume {
        // Reprise depuis une sauvegarde
        let state = load_state(Path::new(path))?;
        // Utiliser le tick actuel comme seed pour le rng (reproductibilite approximative)
        let rng = SeededRng::new(state.tick_count as u64);
        println!("Reprise depuis {}, tick {}", path, state.tick_count);
        (state, rng)
    } else {
        // Nouvelle simulation
        let (config, seed) = load_config(Path::new(config_path))?;
        let pop = config.initial_population;
        let mut rng = SeededRng::new(seed);

        // Generer le terrain Perlin
        let mut world = World::new(DEFAULT_GRID_SIZE);
        let island = generate_island(&mut world, seed as u32, 0.2);
        let state = SimState::with_terrain(world, island, config, &mut rng);

        println!("Simulation demarree (seed: {}, population: {})", seed, pop);
        (state, rng)
    };

    if no_tui {
        run_headless(state, rng)
    } else {
        run_with_tui(state, rng)
    }
}

/// Boucle de simulation avec TUI ratatui (mode par defaut).
fn run_with_tui(state: SimState, rng: SeededRng) -> Result<(), String> {
    let controls = SimControls::new();
    let (tx, rx) = mpsc::channel();

    // Handler Ctrl+C — filet de securite en complement de crossterm
    let quit_signal = controls.quit.clone();
    ctrlc::set_handler(move || {
        quit_signal.store(true, Ordering::Relaxed);
    })
    .map_err(|e| e.to_string())?;

    let sim_handle = spawn_simulation(state, rng, controls.clone(), tx);

    let mut tui = Tui::new().map_err(|e| e.to_string())?;

    let mut last_snapshot = SimSnapshot::default();
    let mut app_mode = AppMode::Dashboard;
    let mut island_layer: u8 = 0;

    loop {
        // Verifier si Ctrl+C a ete recu via le handler
        if controls.quit.load(Ordering::Relaxed) {
            break;
        }

        // Recevoir le dernier snapshot (non bloquant)
        while let Ok(snap) = rx.try_recv() {
            last_snapshot = snap;
        }

        // Injecter le calque actif dans le snapshot
        last_snapshot.active_layer = island_layer;

        // Dessiner avec le mode actif
        tui.draw(&last_snapshot, app_mode)
            .map_err(|e| e.to_string())?;

        // Poll events clavier (timeout 33ms ~ 30fps)
        if event::poll(Duration::from_millis(33)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            controls.quit.store(true, Ordering::Relaxed);
                            break;
                        }
                        KeyCode::Char(' ') => {
                            let current = controls.paused.load(Ordering::Relaxed);
                            controls.paused.store(!current, Ordering::Relaxed);
                        }
                        KeyCode::Char('s') => {
                            controls.save_requested.store(true, Ordering::Relaxed);
                        }
                        // Navigation deep dives (toggle : re-appuyer revient au dashboard)
                        KeyCode::Char('1') => {
                            app_mode = if app_mode == AppMode::Evolution {
                                AppMode::Dashboard
                            } else {
                                AppMode::Evolution
                            };
                        }
                        KeyCode::Char('2') => {
                            app_mode = if app_mode == AppMode::Population {
                                AppMode::Dashboard
                            } else {
                                AppMode::Population
                            };
                        }
                        KeyCode::Char('3') => {
                            app_mode = if app_mode == AppMode::Cooperation {
                                AppMode::Dashboard
                            } else {
                                AppMode::Cooperation
                            };
                        }
                        KeyCode::Char('4') => {
                            if app_mode == AppMode::Island {
                                app_mode = AppMode::Dashboard;
                                controls.island_active.store(false, Ordering::Relaxed);
                            } else {
                                app_mode = AppMode::Island;
                                controls.island_active.store(true, Ordering::Relaxed);
                            }
                        }
                        KeyCode::Char('5') => {
                            app_mode = if app_mode == AppMode::Logs {
                                AppMode::Dashboard
                            } else {
                                AppMode::Logs
                            };
                        }
                        // Calques ile (uniquement en mode Island)
                        KeyCode::Char('a') if app_mode == AppMode::Island => {
                            island_layer = 1;
                            controls.island_layer.store(1, Ordering::Relaxed);
                        }
                        KeyCode::Char('b') if app_mode == AppMode::Island => {
                            island_layer = 2;
                            controls.island_layer.store(2, Ordering::Relaxed);
                        }
                        KeyCode::Char('c') if app_mode == AppMode::Island => {
                            island_layer = 3;
                            controls.island_layer.store(3, Ordering::Relaxed);
                        }
                        KeyCode::Char('d') if app_mode == AppMode::Island => {
                            island_layer = 4;
                            controls.island_layer.store(4, Ordering::Relaxed);
                        }
                        KeyCode::Char('e') if app_mode == AppMode::Island => {
                            island_layer = 5;
                            controls.island_layer.store(5, Ordering::Relaxed);
                        }
                        KeyCode::Char('f') if app_mode == AppMode::Island => {
                            island_layer = 6;
                            controls.island_layer.store(6, Ordering::Relaxed);
                        }
                        KeyCode::Char('0') if app_mode == AppMode::Island => {
                            island_layer = 0;
                            controls.island_layer.store(0, Ordering::Relaxed);
                        }
                        KeyCode::Esc => {
                            // Desactiver le scan ile si on quitte la vue
                            if app_mode == AppMode::Island {
                                controls.island_active.store(false, Ordering::Relaxed);
                            }
                            app_mode = AppMode::Dashboard;
                            island_layer = 0;
                            controls.island_layer.store(0, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Attendre la fin du thread simulation
    if let Err(e) = sim_handle.join() {
        eprintln!("Le thread de simulation a paniqué : {:?}", e);
    }

    // Restaurer le terminal
    tui.restore().map_err(|e| e.to_string())?;

    Ok(())
}

/// Boucle de simulation headless (mode --no-tui) — affichage texte toutes les 100 ticks.
fn run_headless(mut state: SimState, mut rng: SeededRng) -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .map_err(|e| e.to_string())?;

    let mut recorder = ReplayRecorder::new(ReplayConfig::default());

    while running.load(Ordering::Relaxed) {
        let events = run_tick(&mut state, &mut rng);

        // Replay — les highlights sont calcules dans run_tick via state.metrics
        recorder.record_tick(state.tick_count, &events);
        recorder.process_highlights(state.tick_count, &state.metrics.recent_highlights);

        // Auto-save toutes les 1000 ticks
        if should_auto_save(state.tick_count, 1000) {
            let slot = get_auto_save_slot(state.tick_count, 3, 1000);
            auto_save(&state, Path::new("saves"), slot)?;
        }

        // Affichage toutes les 100 ticks
        if state.tick_count.is_multiple_of(100) {
            let season = state.season_cycle.current_season();
            let year = state.season_cycle.year();
            println!(
                "[tick {}] annee {} | {:?} | pop: {} | symbiose: {} | lignees: {} | best fitness: {:.1}",
                state.tick_count,
                year,
                season,
                state.metrics.alive_count,
                state.metrics.symbiosis_count,
                state.metrics.lineage_count,
                state.seed_bank.best_fitness()
            );
        }

        // Sauvegarder le montage toutes les 5000 ticks
        if state.tick_count.is_multiple_of(5000) && state.tick_count > 0 {
            recorder.finalize_clips(state.tick_count);
            if let Err(e) = recorder.save_montage(
                Path::new(&format!("replays/montage_{:06}.json", state.tick_count)),
                state.tick_count,
            ) {
                eprintln!("Erreur sauvegarde montage: {}", e);
            }
        }
    }

    // Sauvegarde finale avant arret
    println!("Sauvegarde finale...");
    let _ = auto_save(&state, Path::new("saves"), 1);
    println!("Simulation arretee. {} ticks effectues.", state.tick_count);

    Ok(())
}

/// Sert le viewer web avec un fichier de montage pour le replay.
fn cmd_replay(montage_path: &str, port: u16) -> Result<(), String> {
    let montage = Path::new(montage_path);
    if !montage.exists() {
        return Err(format!("Fichier montage introuvable: {}", montage_path));
    }
    let web_dir = server::find_web_dir().ok_or("Dossier web/ introuvable")?;
    server::serve_replay(port, montage, &web_dir)
}

/// Lance la simulation en mode live avec WebSocket.
fn cmd_live(config_path: &str, port: u16, ws_port: u16) -> Result<(), String> {
    fs::create_dir_all("saves")
        .map_err(|e| format!("impossible de creer le dossier saves/: {e}"))?;

    let (config, seed) = load_config(Path::new(config_path))?;
    let pop = config.initial_population;
    let mut rng = SeededRng::new(seed);

    let mut world = World::new(DEFAULT_GRID_SIZE);
    let island = generate_island(&mut world, seed as u32, 0.2);
    let state = SimState::with_terrain(world, island, config, &mut rng);

    println!(
        "Simulation live demarree (seed: {}, population: {})",
        seed, pop
    );
    live::run_live(state, rng, port, ws_port)
}

/// Genere un fichier garden.toml par defaut.
fn cmd_config_init() -> Result<(), String> {
    let path = Path::new("garden.toml");

    if path.exists() {
        println!("Attention: garden.toml existe deja, ecrasement...");
    }

    let content = generate_default_toml();
    fs::write(path, content).map_err(|e| format!("impossible d'ecrire garden.toml: {e}"))?;

    println!("Fichier garden.toml cree");
    Ok(())
}

/// Pepiniere TUI — pre-entrainement genetique avec interface ratatui.
fn cmd_nursery_tui(
    config_path: &str,
    generations: u32,
    population: usize,
    seed: u64,
    bank: Option<&str>,
    action: Option<NurseryAction>,
) -> Result<(), String> {
    // 1. Charger les environnements
    let path = Path::new(config_path);
    let envs = garden_core::load_nursery_environments(path)
        .map_err(|e| format!("Erreur chargement config: {e}"))?;

    // 2. Charger la banque si --bank fourni
    let initial_genomes = if let Some(bank_path) = bank {
        let bank_file = Path::new(bank_path);
        let (dto, genomes) = garden_core::load_seed_bank(bank_file)
            .map_err(|e| format!("Erreur chargement banque: {e}"))?;
        let best = dto
            .champions
            .iter()
            .map(|c| c.fitness)
            .fold(0.0_f32, f32::max);
        eprintln!(
            "Reprise depuis {} ({} champions, best: {:.4})",
            bank_path,
            genomes.len(),
            best
        );
        Some(genomes)
    } else {
        None
    };

    // 3. Initialiser le snapshot
    let mut snapshot = NurserySnapshot::new(&envs, generations, population, seed);
    let mut view_mode = NurseryViewMode::Recap;

    // 4. Channel + controls
    let (tx, rx) = mpsc::channel();
    let controls = NurseryControls::new();

    // 5. Spawner le thread nursery
    let handle = spawn_nursery(
        envs.clone(),
        generations,
        population,
        seed,
        initial_genomes,
        controls.clone(),
        tx,
    );

    // 6. Initialiser le TUI
    let mut tui = Tui::new().map_err(|e| e.to_string())?;

    // 7. Event loop
    loop {
        // Poll clavier (33ms ~ 30fps)
        if event::poll(Duration::from_millis(33)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            controls.request_quit();
                            break;
                        }
                        KeyCode::Char('p') => {
                            controls.toggle_pause();
                            snapshot.paused = controls.is_paused();
                        }
                        KeyCode::Char('s') => {
                            // Save seulement si termine
                            if snapshot.finished {
                                if let Some(results) = &snapshot.results {
                                    let save_path = Path::new("seeds/latest.json");
                                    if let Err(e) =
                                        garden_core::export_seed_bank(results, save_path)
                                    {
                                        eprintln!("Erreur save: {e}");
                                    }
                                }
                            }
                        }
                        KeyCode::Up => {
                            if let NurseryViewMode::Recap = &view_mode {
                                if snapshot.selected_env > 0 {
                                    snapshot.selected_env -= 1;
                                } else {
                                    snapshot.selected_env =
                                        snapshot.envs.len().saturating_sub(1);
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let NurseryViewMode::Recap = &view_mode {
                                snapshot.selected_env =
                                    (snapshot.selected_env + 1) % snapshot.envs.len().max(1);
                            }
                        }
                        KeyCode::Enter => {
                            if let NurseryViewMode::Recap = &view_mode {
                                view_mode = NurseryViewMode::Zoom(snapshot.selected_env);
                            }
                        }
                        KeyCode::Esc => {
                            view_mode = NurseryViewMode::Recap;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Drainer les updates du channel
        while let Ok(update) = rx.try_recv() {
            snapshot.apply_update(update);
        }

        // Dessiner
        tui.draw_nursery(&snapshot, &view_mode)
            .map_err(|e| format!("Erreur draw: {e}"))?;
    }

    // 8. Attendre la fin du thread nursery
    let _ = handle.join();

    // 9. Restaurer le terminal
    tui.restore().map_err(|e| e.to_string())?;

    // 10. Afficher le resume en mode texte apres la TUI
    if let Some(results) = &snapshot.results {
        println!("\n--- Resume ---");
        for r in results {
            println!(
                "{:20} | fitness: {:.4} | {} generations",
                r.env_name, r.fitness, r.generations_run
            );
        }

        // Commit si demande
        if let Some(NurseryAction::Commit { output }) = &action {
            let output_path = Path::new(output);
            garden_core::export_seed_bank(results, output_path)
                .map_err(|e| format!("Erreur export: {e}"))?;
            println!(
                "Seed bank exportee vers {} ({} champions)",
                output,
                results.len()
            );
        }
    }

    Ok(())
}

/// Pepiniere headless — pre-entrainement genetique des graines (mode texte).
fn cmd_nursery_headless(
    config_path: &str,
    generations: u32,
    population: usize,
    seed: u64,
    verbose: bool,
    bank: Option<&str>,
    action: Option<NurseryAction>,
) -> Result<(), String> {
    // 1. Charger les environnements
    let path = Path::new(config_path);
    let envs = garden_core::load_nursery_environments(path)
        .map_err(|e| format!("Erreur chargement config: {e}"))?;

    // Charger la banque si --bank fourni
    let initial_genomes = match bank {
        Some(bank_path) => {
            let bank_file = Path::new(bank_path);
            match garden_core::load_seed_bank(bank_file) {
                Ok((dto, genomes)) => {
                    let best = dto
                        .champions
                        .iter()
                        .map(|c| c.fitness)
                        .fold(0.0_f32, f32::max);
                    println!(
                        "Reprise depuis {} ({} champions, best: {:.4})",
                        bank_path,
                        genomes.len(),
                        best
                    );
                    Some(genomes)
                }
                Err(e) => {
                    return Err(format!("Erreur chargement banque: {}", e));
                }
            }
        }
        None => None,
    };

    println!(
        "Pepiniere — {} environnements, {} generations, pop {}, seed {}",
        envs.len(),
        generations,
        population,
        seed
    );

    let multi = envs.len() > 1;

    // 2. Callback d'affichage
    let cb = move |env_name: &str, report: &garden_core::GenerationReport| {
        if multi {
            print!("[{:15}] ", env_name);
        }
        println!(
            "Gen {:4} | best: {:.4} | avg: {:.4} | worst: {:.4} | {:.1}s",
            report.generation,
            report.best_fitness,
            report.avg_fitness,
            report.worst_fitness,
            report.elapsed_secs,
        );

        if verbose {
            if let Some(stats) = &report.champion_stats {
                if multi {
                    print!("{:18}", "");
                }
                println!(
                    "  champion: biomass={} territory={} seeds={} symbiosis={} cn_exchanges={:.1}",
                    stats.max_biomass,
                    stats.max_territory,
                    stats.seeds_produced,
                    stats.symbiotic_connections,
                    stats.cn_exchanges,
                );
            }
        }
    };

    // 3. Lancer la nursery
    let results = garden_core::run_nursery_all(
        &envs,
        generations,
        population,
        seed,
        Some(&cb),
        initial_genomes.as_deref(),
    );

    // 4. Resume final
    println!("\n--- Resume ---");
    for r in &results {
        println!(
            "{:20} | fitness: {:.4} | {} generations",
            r.env_name, r.fitness, r.generations_run
        );
    }

    // 5. Export si commit demande
    if let Some(NurseryAction::Commit { output }) = &action {
        let output_path = Path::new(output);
        match garden_core::export_seed_bank(&results, output_path) {
            Ok(()) => println!(
                "Seed bank exportee vers {} ({} champions)",
                output,
                results.len()
            ),
            Err(e) => {
                return Err(format!("Erreur export: {}", e));
            }
        }
    }

    Ok(())
}
