// CLI pour Neural Garden — lance la simulation ou gere la configuration.

use std::fs;
use std::path::Path;

use clap::{Parser, Subcommand};
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::world::World;
use garden_core::infra::config::{generate_default_toml, load_config};
use garden_core::infra::noise::generate_island;
use garden_core::infra::persistence::{
    auto_save, get_auto_save_slot, load_state, should_auto_save,
};
use garden_core::infra::replay::{ReplayConfig, ReplayRecorder};
use garden_core::infra::rng::SeededRng;

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
    },
    /// Gerer la configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
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
        Commands::Run { config, resume } => cmd_run(&config, resume.as_deref()),
        Commands::Config { action } => match action {
            ConfigAction::Init => cmd_config_init(),
        },
    };

    if let Err(e) = result {
        eprintln!("Erreur: {e}");
        std::process::exit(1);
    }
}

/// Lance la simulation (nouvelle ou reprise).
fn cmd_run(config_path: &str, resume: Option<&str>) -> Result<(), String> {
    // Creer les dossiers necessaires
    fs::create_dir_all("saves")
        .map_err(|e| format!("impossible de creer le dossier saves/: {e}"))?;
    fs::create_dir_all("replays")
        .map_err(|e| format!("impossible de creer le dossier replays/: {e}"))?;

    let (mut state, mut rng) = if let Some(path) = resume {
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

    // Initialiser les outils de suivi
    let mut recorder = ReplayRecorder::new(ReplayConfig::default());

    // Boucle principale
    loop {
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
        if state.tick_count % 100 == 0 {
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
        if state.tick_count % 5000 == 0 && state.tick_count > 0 {
            recorder.finalize_clips(state.tick_count);
            if let Err(e) = recorder.save_montage(
                Path::new(&format!("replays/montage_{:06}.json", state.tick_count)),
                state.tick_count,
            ) {
                eprintln!("Erreur sauvegarde montage: {}", e);
            }
        }
    }
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
