// Mode live : simulation avec serveur WebSocket pour le viewer temps reel.

use std::net::TcpListener;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use tungstenite::accept;

use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::events::DomainEvent;
use garden_core::domain::plant::Pos;
use garden_core::domain::world::GRID_SIZE;
use garden_core::infra::dto::DomainEventDto;
use garden_core::infra::persistence::{auto_save, get_auto_save_slot, should_auto_save};
use garden_core::infra::rng::SeededRng;

use crate::server::{find_web_dir, serve_replay};

/// Lance la simulation en mode live avec serveur WebSocket.
pub fn run_live(
    mut state: SimState,
    mut rng: SeededRng,
    http_port: u16,
    ws_port: u16,
) -> Result<(), String> {
    let quit = Arc::new(AtomicBool::new(false));

    // Handler Ctrl+C
    let quit_signal = quit.clone();
    ctrlc::set_handler(move || {
        quit_signal.store(true, Ordering::Relaxed);
    })
    .map_err(|e| e.to_string())?;

    // Thread serveur HTTP (sert le viewer)
    let web_dir = find_web_dir().ok_or("Dossier web/ introuvable")?;
    let web_dir_clone = web_dir.clone();
    thread::spawn(move || {
        // Pour le mode live, pas de montage.json — le viewer utilise le WebSocket.
        // On sert un fichier inexistant ; le viewer gere le 404.
        let empty_montage = Path::new("/dev/null");
        let _ = serve_replay(http_port, empty_montage, &web_dir_clone);
    });

    println!("Viewer disponible sur http://localhost:{}", http_port);
    println!(
        "  URL complete: http://localhost:{}/?live=ws://localhost:{}",
        http_port, ws_port
    );
    println!("Ctrl+C pour arreter");

    // Serveur WebSocket
    let ws_addr = format!("0.0.0.0:{}", ws_port);
    let listener =
        TcpListener::bind(&ws_addr).map_err(|e| format!("Erreur bind WebSocket: {}", e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Erreur set_nonblocking: {}", e))?;

    // Stocker les senders pour les clients connectes
    let clients: Arc<Mutex<Vec<mpsc::Sender<String>>>> = Arc::new(Mutex::new(Vec::new()));
    let clients_accept = clients.clone();

    // Dernier snapshot JSON, partage entre le thread d'acceptation et la boucle de simulation
    let last_snapshot_json: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let last_snapshot_accept = last_snapshot_json.clone();

    // Thread pour accepter les nouvelles connexions WebSocket
    let quit_accept = quit.clone();
    thread::spawn(move || {
        accept_loop(
            &listener,
            &clients_accept,
            &quit_accept,
            &last_snapshot_accept,
        );
    });

    // Boucle de simulation
    while !quit.load(Ordering::Relaxed) {
        // Executer un tick
        let events = run_tick(&mut state, &mut rng);

        // Auto-save toutes les 1000 ticks
        if should_auto_save(state.tick_count, 1000) {
            let slot = get_auto_save_slot(state.tick_count, 3, 1000);
            let _ = auto_save(&state, Path::new("saves"), slot);
        }

        // Mettre a jour le dernier snapshot pour les nouveaux clients
        {
            let snapshot = build_initial_snapshot(&state);
            if let (Ok(json), Ok(mut cached)) =
                (serde_json::to_string(&snapshot), last_snapshot_json.lock())
            {
                *cached = json;
            }
        }

        // Envoyer les events aux clients
        let tick_msg = build_tick_message(&state, &events);
        let json = serde_json::to_string(&tick_msg).unwrap_or_default();

        if let Ok(mut client_list) = clients.lock() {
            // Retirer les clients deconnectes (send a echoue)
            client_list.retain(|client| client.send(json.clone()).is_ok());
        }

        // Throttle : limiter a ~30 msgs/s
        if state.tick_count.is_multiple_of(3) {
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    // Sauvegarde finale
    let _ = auto_save(&state, Path::new("saves"), 1);
    println!("Simulation arretee. {} ticks effectues.", state.tick_count);

    Ok(())
}

/// Boucle d'acceptation des connexions WebSocket.
fn accept_loop(
    listener: &TcpListener,
    clients: &Arc<Mutex<Vec<mpsc::Sender<String>>>>,
    quit: &Arc<AtomicBool>,
    last_snapshot_json: &Arc<Mutex<String>>,
) {
    loop {
        if quit.load(Ordering::Relaxed) {
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let ws = match accept(stream) {
                    Ok(ws) => ws,
                    Err(_) => continue,
                };

                let (client_tx, client_rx) = mpsc::channel::<String>();

                // Envoyer le dernier snapshot au nouveau client
                if let Ok(snapshot) = last_snapshot_json.lock() {
                    if !snapshot.is_empty() {
                        let _ = client_tx.send(snapshot.clone());
                    }
                }

                // Ajouter le client
                if let Ok(mut list) = clients.lock() {
                    list.push(client_tx);
                }

                // Thread d'envoi pour ce client
                thread::spawn(move || {
                    let mut ws = ws;
                    while let Ok(msg) = client_rx.recv() {
                        if ws.send(tungstenite::Message::Text(msg)).is_err() {
                            break;
                        }
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(_) => break,
        }
    }
}

// --- Structures de message JSON ---

/// Snapshot initial envoye a la connexion WebSocket.
#[derive(serde::Serialize)]
struct InitialSnapshot {
    #[serde(rename = "type")]
    msg_type: String,
    grid_size: u16,
    terrain_heights: Vec<Vec<f32>>,
    plants: Vec<LivePlant>,
    links: Vec<LiveLink>,
    season: String,
    best_fitness: f32,
    tick: u32,
}

#[derive(serde::Serialize)]
struct LivePlant {
    id: u64,
    lineage_id: u64,
    cells: Vec<[u16; 2]>,
    vitality: f32,
    energy: f32,
    biomass: u16,
    state: String,
}

#[derive(serde::Serialize)]
struct LiveLink {
    plant_a: u64,
    plant_b: u64,
    pos_a: Option<[u16; 2]>,
    pos_b: Option<[u16; 2]>,
}

#[derive(serde::Serialize)]
struct TickMessage {
    #[serde(rename = "type")]
    msg_type: String,
    tick: u32,
    season: String,
    best_fitness: f32,
    population: usize,
    events: Vec<serde_json::Value>,
}

// --- Construction des messages ---

fn build_initial_snapshot(state: &SimState) -> InitialSnapshot {
    let grid_size = GRID_SIZE;

    // Extraire les altitudes du terrain
    let mut terrain_heights = vec![vec![0.0f32; grid_size as usize]; grid_size as usize];
    for y in 0..grid_size {
        for x in 0..grid_size {
            let pos = Pos { x, y };
            if let Some(cell) = state.world.get(&pos) {
                terrain_heights[y as usize][x as usize] = cell.altitude();
            }
        }
    }

    // Plantes vivantes
    let plants: Vec<LivePlant> = state
        .plants
        .iter()
        .filter(|p| !p.is_dead())
        .map(|p| LivePlant {
            id: p.id(),
            lineage_id: p.lineage().id(),
            cells: p.canopy().iter().map(|pos| [pos.x, pos.y]).collect(),
            vitality: p.vitality().value(),
            energy: p.energy().value(),
            biomass: p.biomass().value(),
            state: format!("{:?}", p.state()),
        })
        .collect();

    // Liens mycorhiziens
    let links: Vec<LiveLink> = state
        .symbiosis
        .links()
        .iter()
        .map(|l| {
            let pos_a = state
                .plants
                .iter()
                .find(|p| p.id() == l.plant_a())
                .and_then(|p| p.canopy().first())
                .map(|pos| [pos.x, pos.y]);
            let pos_b = state
                .plants
                .iter()
                .find(|p| p.id() == l.plant_b())
                .and_then(|p| p.canopy().first())
                .map(|pos| [pos.x, pos.y]);
            LiveLink {
                plant_a: l.plant_a(),
                plant_b: l.plant_b(),
                pos_a,
                pos_b,
            }
        })
        .collect();

    InitialSnapshot {
        msg_type: "snapshot".to_string(),
        grid_size,
        terrain_heights,
        plants,
        links,
        season: format!("{:?}", state.season_cycle.current_season()),
        best_fitness: state.seed_bank.best_fitness(),
        tick: state.tick_count,
    }
}

fn build_tick_message(state: &SimState, events: &[DomainEvent]) -> TickMessage {
    let event_values: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            let dto = DomainEventDto::from_event(state.tick_count, e);
            serde_json::to_value(&dto).unwrap_or(serde_json::Value::Null)
        })
        .collect();

    TickMessage {
        msg_type: "tick".to_string(),
        tick: state.tick_count,
        season: format!("{:?}", state.season_cycle.current_season()),
        best_fitness: state.seed_bank.best_fitness(),
        population: state.metrics.alive_count,
        events: event_values,
    }
}
