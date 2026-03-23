// Runner multi-thread : la nursery tourne dans un thread separe
// et communique avec le thread UI via un channel mpsc.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;

use garden_core::application::evolution::Genome;
use garden_core::application::nursery::{BedConfig, GenerationReport};
use garden_core::run_nursery_all;

use crate::nursery_snapshot::NurseryUpdate;

/// Controles de la nursery partages entre threads.
#[derive(Clone)]
pub struct NurseryControls {
    /// Paire (Mutex<bool>, Condvar) pour le mecanisme pause/reprise.
    pub paused: Arc<(Mutex<bool>, Condvar)>,
    /// Signal d'arret.
    pub quit: Arc<AtomicBool>,
}

impl NurseryControls {
    pub fn new() -> Self {
        Self {
            paused: Arc::new((Mutex::new(false), Condvar::new())),
            quit: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Bascule l'etat pause/reprise.
    pub fn toggle_pause(&self) {
        let (lock, cvar) = &*self.paused;
        let mut paused = lock.lock().unwrap_or_else(|e| e.into_inner());
        *paused = !*paused;
        if !*paused {
            cvar.notify_all();
        }
    }

    /// Verifie si la nursery est en pause.
    pub fn is_paused(&self) -> bool {
        let (lock, _) = &*self.paused;
        *lock.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Attend si en pause (appele depuis le thread nursery).
    pub fn wait_if_paused(&self) {
        let (lock, cvar) = &*self.paused;
        let mut paused = lock.lock().unwrap_or_else(|e| e.into_inner());
        while *paused {
            paused = cvar.wait(paused).unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Demande l'arret.
    pub fn request_quit(&self) {
        self.quit.store(true, Ordering::Relaxed);
        // Debloquer si en pause
        let (lock, cvar) = &*self.paused;
        let mut paused = lock.lock().unwrap_or_else(|e| e.into_inner());
        *paused = false;
        cvar.notify_all();
        drop(paused);
    }
}

/// Lance la nursery dans un thread separe.
/// Les updates sont envoyees via le channel tx.
pub fn spawn_nursery(
    envs: Vec<(String, BedConfig)>,
    generations: u32,
    population: usize,
    seed: u64,
    initial_genomes: Option<Vec<Genome>>,
    controls: NurseryControls,
    tx: mpsc::Sender<NurseryUpdate>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let controls_cb = controls.clone();
        let tx_cb = tx.clone();

        // Callback appele a chaque generation par run_nursery_all
        let cb = move |env_name: &str, report: &GenerationReport| {
            // Verifier arret demande
            if controls_cb.quit.load(Ordering::Relaxed) {
                return;
            }

            // Verifier pause
            controls_cb.wait_if_paused();

            // Envoyer l'update (ignorer l'erreur si le receiver est ferme)
            let _ = tx_cb.send(NurseryUpdate::Generation {
                env_name: env_name.to_string(),
                report: report.clone(),
            });
        };

        let results = run_nursery_all(
            &envs,
            generations,
            population,
            seed,
            Some(&cb),
            initial_genomes.as_deref(),
        );

        let _ = tx.send(NurseryUpdate::Finished { results });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn controls_commence_non_pause() {
        let controls = NurseryControls::new();
        assert!(!controls.is_paused());
    }

    #[test]
    fn toggle_pause_bascule_l_etat() {
        let controls = NurseryControls::new();

        controls.toggle_pause();
        assert!(controls.is_paused());

        controls.toggle_pause();
        assert!(!controls.is_paused());
    }

    #[test]
    fn request_quit_debloque_la_pause() {
        let controls = NurseryControls::new();

        // Mettre en pause
        controls.toggle_pause();
        assert!(controls.is_paused());

        // Demander l'arret — doit debloquer la pause
        controls.request_quit();
        assert!(!controls.is_paused());
        assert!(controls.quit.load(Ordering::Relaxed));
    }
}
