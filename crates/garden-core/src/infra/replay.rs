// Enregistreur de replay : serialisation des events et filtrage des highlights.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::application::highlights::Highlight;
use crate::domain::events::DomainEvent;

use super::dto::DomainEventDto;

/// Configuration du filtre de replay.
pub struct ReplayConfig {
    /// Cooldown minimum entre deux clips (en ticks).
    pub min_cooldown: u32,
    /// Score minimum pour qu'un highlight devienne un clip.
    pub min_score: f32,
    /// Nombre maximum de clips dans un montage.
    pub max_clips: usize,
    /// Nombre de ticks avant le trigger a capturer.
    pub capture_before: u32,
    /// Nombre de ticks apres le trigger a capturer.
    pub capture_after: u32,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            min_cooldown: 200,
            min_score: 0.3,
            max_clips: 20,
            capture_before: 75,
            capture_after: 75,
        }
    }
}

/// Un clip capture autour d'un moment cle.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplayClip {
    pub trigger: String,
    pub tick_start: u32,
    pub tick_end: u32,
    pub score: f32,
    pub events: Vec<DomainEventDto>,
}

/// Montage complet : collection de clips.
#[derive(Serialize, Deserialize, Debug)]
pub struct ReplayMontage {
    pub version: u32,
    pub total_ticks: u32,
    pub clips: Vec<ReplayClip>,
}

/// Etat interne d'un clip en cours de capture (events "apres" pas encore recus).
struct PendingClip {
    trigger: String,
    tick_start: u32,
    tick_end: u32,
    score: f32,
    events: Vec<DomainEventDto>,
}

/// Enregistreur de replay avec buffer circulaire d'events.
pub struct ReplayRecorder {
    config: ReplayConfig,
    /// Buffer circulaire des events recents (pour capturer les events "avant" le trigger).
    event_buffer: Vec<(u32, Vec<DomainEventDto>)>,
    /// Clips finalises.
    clips: Vec<ReplayClip>,
    /// Clips en cours de capture (attendent les events "apres").
    pending_clips: Vec<PendingClip>,
    /// Tick du dernier clip capture (pour le cooldown).
    last_clip_tick: Option<u32>,
}

impl ReplayRecorder {
    /// Cree un nouveau recorder avec la configuration donnee.
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            config,
            event_buffer: Vec::new(),
            clips: Vec::new(),
            pending_clips: Vec::new(),
            last_clip_tick: None,
        }
    }

    /// Enregistre les events d'un tick dans le buffer circulaire.
    pub fn record_tick(&mut self, tick: u32, events: &[DomainEvent]) {
        // Convertir les events en DTO
        let dtos: Vec<DomainEventDto> = events
            .iter()
            .map(|e| DomainEventDto::from_event(tick, e))
            .collect();

        // Ajouter au buffer
        self.event_buffer.push((tick, dtos.clone()));

        // Garder seulement les derniers capture_before ticks
        let max_buffer = self.config.capture_before as usize;
        if self.event_buffer.len() > max_buffer {
            let drain_count = self.event_buffer.len() - max_buffer;
            self.event_buffer.drain(..drain_count);
        }

        // Alimenter les clips en cours de capture avec les events de ce tick
        for pending in &mut self.pending_clips {
            if tick <= pending.tick_end {
                pending.events.extend(dtos.clone());
            }
        }
    }

    /// Traite les highlights detectes et demarre la capture de clips si pertinent.
    pub fn process_highlights(&mut self, tick: u32, highlights: &[Highlight]) {
        for highlight in highlights {
            // Filtrer par score minimum
            if highlight.score < self.config.min_score {
                continue;
            }

            // Filtrer par cooldown
            if let Some(last_tick) = self.last_clip_tick {
                if last_tick + self.config.min_cooldown > tick {
                    continue;
                }
            }

            // Filtrer par budget max
            let total_clips = self.clips.len() + self.pending_clips.len();
            if total_clips >= self.config.max_clips {
                continue;
            }

            // Calculer les bornes du clip
            let tick_start = tick.saturating_sub(self.config.capture_before);
            let tick_end = tick + self.config.capture_after;

            // Collecter les events du buffer (les ticks "avant")
            let buffer_events: Vec<DomainEventDto> = self
                .event_buffer
                .iter()
                .filter(|(t, _)| *t >= tick_start)
                .flat_map(|(_, dtos): &(u32, Vec<DomainEventDto>)| dtos.clone())
                .collect();

            // Creer un clip en attente
            let trigger = format!("{:?}", highlight.highlight_type);
            self.pending_clips.push(PendingClip {
                trigger,
                tick_start,
                tick_end,
                score: highlight.score,
                events: buffer_events,
            });

            self.last_clip_tick = Some(tick);
        }
    }

    /// Finalise les clips dont la fenetre de capture est terminee.
    pub fn finalize_clips(&mut self, tick: u32) {
        let mut finalized = Vec::new();
        let mut remaining = Vec::new();

        for pending in self.pending_clips.drain(..) {
            if tick >= pending.tick_end {
                finalized.push(ReplayClip {
                    trigger: pending.trigger,
                    tick_start: pending.tick_start,
                    tick_end: pending.tick_end,
                    score: pending.score,
                    events: pending.events,
                });
            } else {
                remaining.push(pending);
            }
        }

        self.clips.extend(finalized);
        self.pending_clips = remaining;
    }

    /// Sauvegarde le montage dans un fichier JSON.
    pub fn save_montage(&self, path: &Path, total_ticks: u32) -> Result<(), String> {
        // Finaliser aussi les clips pending dans le montage
        let mut all_clips = self.clips.clone();
        for pending in &self.pending_clips {
            all_clips.push(ReplayClip {
                trigger: pending.trigger.clone(),
                tick_start: pending.tick_start,
                tick_end: pending.tick_end,
                score: pending.score,
                events: pending.events.clone(),
            });
        }

        let montage = ReplayMontage {
            version: 1,
            total_ticks,
            clips: all_clips,
        };

        let json = serde_json::to_string_pretty(&montage)
            .map_err(|e| format!("Erreur de serialisation JSON : {e}"))?;

        fs::write(path, json).map_err(|e| format!("Erreur d'ecriture du fichier : {e}"))?;

        Ok(())
    }

    /// Nombre de clips captures (finalises + en cours).
    pub fn clip_count(&self) -> usize {
        self.clips.len() + self.pending_clips.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::highlights::{Highlight, HighlightType};
    use crate::domain::plant::Pos;

    /// Helper : cree un recorder avec config par defaut.
    fn recorder() -> ReplayRecorder {
        ReplayRecorder::new(ReplayConfig::default())
    }

    /// Helper : cree un highlight avec le score donne.
    fn highlight_with_score(tick: u32, score: f32) -> Highlight {
        Highlight {
            tick,
            highlight_type: HighlightType::FirstSymbiosis,
            score,
        }
    }

    #[test]
    fn le_recorder_filtre_par_score() {
        let mut rec = recorder();

        // Enregistrer quelques ticks
        let events = vec![DomainEvent::Linked {
            plant_a: 1,
            plant_b: 2,
        }];
        rec.record_tick(100, &events);

        // Highlight avec score trop bas (0.1 < 0.3)
        let highlights = vec![highlight_with_score(100, 0.1)];
        rec.process_highlights(100, &highlights);

        assert_eq!(rec.clip_count(), 0);
    }

    #[test]
    fn le_recorder_respecte_le_cooldown() {
        let mut rec = recorder();

        // Premier highlight au tick 100
        rec.record_tick(100, &[]);
        let h1 = vec![highlight_with_score(100, 0.8)];
        rec.process_highlights(100, &h1);
        assert_eq!(rec.clip_count(), 1);

        // Deuxieme highlight au tick 150 (50 ticks apres, < 200 de cooldown)
        rec.record_tick(150, &[]);
        let h2 = vec![highlight_with_score(150, 0.9)];
        rec.process_highlights(150, &h2);

        // Un seul clip car le cooldown n'est pas respecte
        assert_eq!(rec.clip_count(), 1);
    }

    #[test]
    fn le_recorder_capture_un_clip() {
        let mut rec = recorder();

        // Enregistrer des events
        let events = vec![DomainEvent::Grew {
            plant_id: 1,
            cell: Pos { x: 5, y: 5 },
            layer: crate::domain::events::GrowthLayer::Footprint,
        }];
        rec.record_tick(500, &events);

        // Highlight avec bon score
        let highlights = vec![highlight_with_score(500, 0.8)];
        rec.process_highlights(500, &highlights);

        assert_eq!(rec.clip_count(), 1);
    }

    #[test]
    fn le_montage_se_serialise_en_json() {
        let mut rec = recorder();

        // Enregistrer un tick et capturer un clip
        let events = vec![DomainEvent::Linked {
            plant_a: 1,
            plant_b: 2,
        }];
        rec.record_tick(300, &events);

        let highlights = vec![highlight_with_score(300, 0.8)];
        rec.process_highlights(300, &highlights);

        // Finaliser (simuler la fin de la fenetre de capture)
        rec.finalize_clips(300 + 75);

        // Sauvegarder dans un fichier temporaire
        let dir = std::env::temp_dir().join("neural_garden_test_replay");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_montage.json");

        let result = rec.save_montage(&path, 1000);
        assert!(result.is_ok(), "save_montage a echoue : {:?}", result);

        // Relire et verifier que c'est du JSON valide
        let contents = fs::read_to_string(&path).expect("Impossible de lire le fichier");
        let montage: Result<ReplayMontage, _> = serde_json::from_str(&contents);
        assert!(montage.is_ok(), "JSON invalide : {:?}", montage);

        let montage = montage.expect("deja verifie");
        assert_eq!(montage.version, 1);
        assert_eq!(montage.total_ticks, 1000);
        assert!(!montage.clips.is_empty());

        // Nettoyage
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
