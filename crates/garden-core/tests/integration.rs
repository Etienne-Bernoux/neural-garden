use cucumber::World;
use garden_core::application::sim::SimState;
use garden_core::domain::rng::Rng;

mod steps;

#[derive(World)]
pub struct GardenWorld {
    pub state: Option<SimState>,
    pub rng: TestRng,
    /// Valeurs capturees pour les assertions
    pub captured_carbon_before: f32,
    pub captured_energy_a: f32,
    pub captured_energy_b: f32,
    pub captured_biomass_summer: u16,
    pub captured_biomass_winter: u16,
    pub captured_carbon_spring: f32,
    pub captured_carbon_winter: f32,
}

impl std::fmt::Debug for GardenWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GardenWorld")
            .field("state", &self.state.as_ref().map(|_| "<SimState>"))
            .field("rng", &self.rng)
            .field("captured_carbon_before", &self.captured_carbon_before)
            .field("captured_biomass_summer", &self.captured_biomass_summer)
            .field("captured_biomass_winter", &self.captured_biomass_winter)
            .field("captured_carbon_spring", &self.captured_carbon_spring)
            .field("captured_carbon_winter", &self.captured_carbon_winter)
            .finish()
    }
}

/// Rng de test simple.
#[derive(Debug)]
pub struct TestRng {
    value: f32,
}

impl TestRng {
    pub fn new() -> Self {
        Self { value: 0.42 }
    }
}

impl Rng for TestRng {
    fn next_f32(&mut self) -> f32 {
        let v = self.value;
        self.value = (self.value + 0.07) % 1.0;
        v
    }
}

impl Default for GardenWorld {
    fn default() -> Self {
        Self {
            state: None,
            rng: TestRng::new(),
            captured_carbon_before: 0.0,
            captured_energy_a: 0.0,
            captured_energy_b: 0.0,
            captured_biomass_summer: 0,
            captured_biomass_winter: 0,
            captured_carbon_spring: 0.0,
            captured_carbon_winter: 0.0,
        }
    }
}

#[tokio::main]
async fn main() {
    GardenWorld::run("tests/features").await;
}
