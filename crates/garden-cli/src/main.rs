// CLI pour Neural Garden — lance la simulation ou gere la configuration.

mod live;
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
use garden_core::domain::world::World;
use garden_core::infra::config::{generate_default_toml, load_config};
use garden_core::infra::noise::generate_island;
use garden_core::infra::persistence::{
    auto_save, get_auto_save_slot, load_state, should_auto_save,
};
use garden_core::infra::replay::{ReplayConfig, ReplayRecorder};
use garden_core::infra::rng::SeededRng;

use crate::runner::{spawn_simulation, SimControls};
use crate::snapshot::SimSnapshot;
use crate::tui::Tui;

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
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Generer un fichier de configuration par defaut
    Init,
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
        let mut world = World::new();
        let island = generate_island(&mut world, seed as u32, 0.3);
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

    loop {
        // Verifier si Ctrl+C a ete recu via le handler
        if controls.quit.load(Ordering::Relaxed) {
            break;
        }

        // Recevoir le dernier snapshot (non bloquant)
        while let Ok(snap) = rx.try_recv() {
            last_snapshot = snap;
        }

        // Dessiner
        tui.draw(&last_snapshot).map_err(|e| e.to_string())?;

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
                "[tick {}] annee {} | {:?} | pop: {} | best fitness: {:.1}",
                state.tick_count,
                year,
                season,
                state.metrics.alive_count,
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

    let mut world = World::new();
    let island = generate_island(&mut world, seed as u32, 0.3);
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
