/// Saison du jardin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// Multiplicateurs saisonniers appliques a l'environnement.
#[derive(Debug, Clone, Copy)]
pub struct SeasonModifiers {
    pub light: f32,
    pub rain: f32,
    pub soil_regen: f32,
    pub growth: f32,
}

impl Season {
    /// Retourne les modificateurs associes a la saison.
    pub fn modifiers(&self) -> SeasonModifiers {
        match self {
            Season::Spring => SeasonModifiers {
                light: 0.8,
                rain: 1.2,
                soil_regen: 1.5,
                growth: 1.3,
            },
            Season::Summer => SeasonModifiers {
                light: 1.0,
                rain: 0.5,
                soil_regen: 0.8,
                growth: 1.0,
            },
            Season::Autumn => SeasonModifiers {
                light: 0.6,
                rain: 1.0,
                soil_regen: 1.2,
                growth: 0.6,
            },
            Season::Winter => SeasonModifiers {
                light: 0.3,
                rain: 0.8,
                soil_regen: 0.5,
                growth: 0.2,
            },
        }
    }
}

/// Gestion du cycle saisonnier.
pub struct SeasonCycle {
    current_tick: u32,
    ticks_per_season: u32,
}

impl SeasonCycle {
    /// Cree un nouveau cycle avec le nombre de ticks par saison specifie.
    pub fn new(ticks_per_season: u32) -> Self {
        Self {
            current_tick: 0,
            ticks_per_season,
        }
    }

    /// Calcule la saison courante depuis le tick.
    pub fn current_season(&self) -> Season {
        let cycle_length = self.ticks_per_season * 4;
        let position = self.current_tick % cycle_length;
        let season_index = position / self.ticks_per_season;
        match season_index {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            3 => Season::Winter,
            _ => unreachable!(),
        }
    }

    /// Avance d'un tick. Retourne Some(nouvelle_saison) si on change de saison.
    pub fn advance(&mut self) -> Option<Season> {
        let old_season = self.current_season();
        self.current_tick += 1;
        let new_season = self.current_season();
        if old_season != new_season {
            Some(new_season)
        } else {
            None
        }
    }

    /// Raccourci pour obtenir les modificateurs de la saison courante.
    pub fn current_modifiers(&self) -> SeasonModifiers {
        self.current_season().modifiers()
    }

    /// Retourne le tick courant.
    pub fn tick(&self) -> u32 {
        self.current_tick
    }

    /// Retourne l'annee courante (nombre de cycles complets).
    pub fn year(&self) -> u32 {
        self.current_tick / (self.ticks_per_season * 4)
    }

    /// Reconstruit un cycle a partir de ses champs bruts.
    /// Utilise pour la deserialisation.
    pub(crate) fn from_raw(current_tick: u32, ticks_per_season: u32) -> Self {
        Self {
            current_tick,
            ticks_per_season,
        }
    }

    /// Retourne le nombre de ticks par saison.
    pub fn ticks_per_season(&self) -> u32 {
        self.ticks_per_season
    }
}

impl Default for SeasonCycle {
    fn default() -> Self {
        Self::new(250)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_cycle_demarre_au_printemps() {
        let cycle = SeasonCycle::default();
        assert_eq!(cycle.current_season(), Season::Spring);
    }

    #[test]
    fn les_saisons_se_succedent() {
        let mut cycle = SeasonCycle::default();

        // Avancer 250 ticks → Summer
        for _ in 0..250 {
            cycle.advance();
        }
        assert_eq!(cycle.current_season(), Season::Summer);

        // Avancer jusqu'a 500 → Autumn
        for _ in 250..500 {
            cycle.advance();
        }
        assert_eq!(cycle.current_season(), Season::Autumn);

        // Avancer jusqu'a 750 → Winter
        for _ in 500..750 {
            cycle.advance();
        }
        assert_eq!(cycle.current_season(), Season::Winter);

        // Avancer jusqu'a 1000 → Spring (nouveau cycle)
        for _ in 750..1000 {
            cycle.advance();
        }
        assert_eq!(cycle.current_season(), Season::Spring);
    }

    #[test]
    fn advance_detecte_la_transition() {
        let mut cycle = SeasonCycle::default();

        // Les 249 premiers advance ne declenchent pas de transition
        for i in 0..249 {
            assert!(
                cycle.advance().is_none(),
                "Transition inattendue au tick {}",
                i + 1
            );
        }

        // Le 250eme declenche la transition vers Summer
        assert_eq!(cycle.advance(), Some(Season::Summer));
    }

    #[test]
    fn les_modificateurs_du_printemps() {
        let mods = Season::Spring.modifiers();
        assert!((mods.light - 0.8).abs() < f32::EPSILON);
        assert!((mods.rain - 1.2).abs() < f32::EPSILON);
        assert!((mods.soil_regen - 1.5).abs() < f32::EPSILON);
        assert!((mods.growth - 1.3).abs() < f32::EPSILON);
    }

    #[test]
    fn les_modificateurs_de_lhiver() {
        let mods = Season::Winter.modifiers();
        assert!((mods.light - 0.3).abs() < f32::EPSILON);
        assert!((mods.rain - 0.8).abs() < f32::EPSILON);
        assert!((mods.soil_regen - 0.5).abs() < f32::EPSILON);
        assert!((mods.growth - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn annee_correcte() {
        let mut cycle = SeasonCycle::default();
        assert_eq!(cycle.year(), 0);

        // Avancer jusqu'au tick 1000 → year 1
        for _ in 0..1000 {
            cycle.advance();
        }
        assert_eq!(cycle.year(), 1);

        // Avancer jusqu'au tick 2500 → year 2
        for _ in 1000..2500 {
            cycle.advance();
        }
        assert_eq!(cycle.year(), 2);
    }
}
