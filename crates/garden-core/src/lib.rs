pub mod application;
pub mod domain;
pub mod infra;

// Facade publique — types principaux
pub use application::sim::{run_tick, SimState};
pub use application::config::SimConfig;
pub use application::season::{Season, SeasonCycle};
pub use application::evolution::{GenerationCounter, Genome, PlantStats, SeedBank};
pub use application::metrics::SimMetrics;
pub use application::highlights::{Highlight, HighlightDetector, HighlightType};

pub use domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, PlantState, Pos};
pub use domain::brain::Brain;
pub use domain::world::{Cell, World, GRID_SIZE};
pub use domain::island::Island;
pub use domain::symbiosis::{MycorrhizalLink, SymbiosisNetwork};
pub use domain::events::{DomainEvent, GrowthLayer};
pub use domain::rng::Rng;

pub use infra::rng::SeededRng;
pub use infra::noise::generate_island;
pub use infra::config::{generate_default_toml, load_config};
pub use infra::persistence::{
    auto_save, get_auto_save_slot, load_state, save_state, should_auto_save,
};
pub use infra::replay::{ReplayConfig, ReplayRecorder};
