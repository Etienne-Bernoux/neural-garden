import * as THREE from 'three';
import { createTerrain } from './infra/terrain.js';
import { PlantRenderer } from './infra/plants.js';
import { InteractionRenderer } from './infra/symbiosis.js';
import { SimState } from './domain/state.js';
import { ClipManager } from './domain/clips.js';
import { Timeline } from './application/timeline.js';
import { GodCamera } from './infra/camera.js';
import { LightingManager } from './infra/lighting.js';
import { PanelManager } from './ui/panel.js';

// --- Init ---
const canvas = document.getElementById('canvas');
const scene = new THREE.Scene();
scene.background = new THREE.Color(0x1a6b8a);  // meme couleur que l'eau

const godCamera = new GodCamera(canvas);
const camera = godCamera.getCamera();

const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setSize(canvas.clientWidth, canvas.clientHeight);
renderer.shadowMap.enabled = true;

const lighting = new LightingManager(scene);
const simState = new SimState();
const plantRenderer = new PlantRenderer(scene, simState.gridSize);
const interactionRenderer = new InteractionRenderer(scene, simState.gridSize);
const panel = new PanelManager();
const clipManager = new ClipManager();
const timeline = new Timeline();

let terrainGroup = null;
let selectedPlantId = null;

// --- Detection du mode ---
// Le viewer peut fonctionner en mode replay (JSON) ou live (WebSocket)
const urlParams = new URLSearchParams(window.location.search);
const montageUrl = urlParams.get('montage') || 'montage.json';
const liveWs = urlParams.get('live');  // ex: ws://localhost:8080

// --- Chargement ---
async function init() {
    if (liveWs) {
        initLive(liveWs);
    } else {
        await loadMontage(montageUrl);
    }
}

async function loadMontage(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            console.warn('Pas de montage disponible');
            panel.updateClipInfo('Aucun montage');
            return;
        }
        const montage = await response.json();

        clipManager.loadMontage(montage);
        loadClip(clipManager.currentClip());
        panel.updateClipInfo(clipManager.clipInfo());
    } catch (e) {
        console.error('Erreur chargement montage:', e);
        panel.updateClipInfo('Erreur chargement');
    }
}

function loadClip(clip) {
    if (!clip) return;

    simState.loadHeader(clip.header || {});
    timeline.loadClip(clip);

    // Terrain
    if (terrainGroup) {
        scene.remove(terrainGroup);
    }
    terrainGroup = createTerrain(clip.header || {});
    scene.add(terrainGroup);

    // Injecter les hauteurs du terrain dans le plant renderer
    plantRenderer.setTerrainHeights(simState.terrainHeights);

    // Eclairage initial
    lighting.setSeason(simState.season);

    // Rendu initial des plantes
    plantRenderer.update(simState.getAlivePlants());
    panel.updateStats(simState);

    // Auto-play
    timeline.play();
}

function initLive(wsUrl) {
    let ws;
    let reconnectDelay = 1000;

    function connect() {
        ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            console.log('WebSocket connecte');
            reconnectDelay = 1000;  // reset le delai
        };

        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);

            if (data.type === 'snapshot') {
                // Snapshot initial
                simState.loadSnapshot(data);

                if (terrainGroup) scene.remove(terrainGroup);
                terrainGroup = createTerrain({ grid_size: data.grid_size, altitude: data.terrain_heights });
                scene.add(terrainGroup);

                plantRenderer.setTerrainHeights(simState.terrainHeights);
            } else if (data.type === 'tick') {
                // Events incrementaux
                if (data.events) {
                    for (const e of data.events) {
                        simState.applyEvent(e);
                    }
                }
                if (data.season && data.season !== simState.season) {
                    simState.season = data.season;
                    lighting.setSeason(simState.season);
                }
                if (data.best_fitness !== undefined) simState.bestFitness = data.best_fitness;
            }

            // Mettre a jour le rendu (eclairage seulement sur snapshot)
            if (data.type === 'snapshot') {
                lighting.setSeason(simState.season);
            }
            plantRenderer.update(simState.getAlivePlants());
            interactionRenderer.updateLinks(simState.links, simState.terrainHeights);
            panel.updateStats(simState);
        };

        ws.onclose = () => {
            console.log(`WebSocket ferme, reconnexion dans ${reconnectDelay/1000}s...`);
            setTimeout(connect, reconnectDelay);
            reconnectDelay = Math.min(reconnectDelay * 2, 10000);  // backoff exponentiel, max 10s
        };

        ws.onerror = (e) => {
            console.error('WebSocket erreur:', e);
            ws.close();  // trigger onclose -> reconnexion
        };
    }

    connect();
}

// --- Controles ---
document.getElementById('btn-play')?.addEventListener('click', () => {
    timeline.togglePlay();
    document.getElementById('btn-play').textContent = timeline.playing ? '\u23F8' : '\u25B6';
});

document.getElementById('btn-prev')?.addEventListener('click', () => {
    const clip = clipManager.prevClip();
    loadClip(clip);
    panel.updateClipInfo(clipManager.clipInfo());
});

document.getElementById('btn-next')?.addEventListener('click', () => {
    const clip = clipManager.nextClip();
    loadClip(clip);
    panel.updateClipInfo(clipManager.clipInfo());
});

document.getElementById('scrub')?.addEventListener('input', (e) => {
    timeline.scrubTo(e.target.value / 100);
});

// Selection de plante par clic
const raycaster = new THREE.Raycaster();
const mouse = new THREE.Vector2();

canvas.addEventListener('click', (e) => {
    const rect = canvas.getBoundingClientRect();
    mouse.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
    mouse.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;

    raycaster.setFromCamera(mouse, camera);
    selectedPlantId = plantRenderer.getPlantAtRaycast(raycaster);

    if (selectedPlantId) {
        const plant = simState.plants.get(selectedPlantId);
        panel.selectPlant(plant || null);
    } else {
        panel.selectPlant(null);
    }
});

// Clavier
document.addEventListener('keydown', (e) => {
    switch (e.key) {
        case ' ':
            e.preventDefault();
            timeline.togglePlay();
            break;
        case 'ArrowLeft':
            loadClip(clipManager.prevClip());
            panel.updateClipInfo(clipManager.clipInfo());
            break;
        case 'ArrowRight':
            loadClip(clipManager.nextClip());
            panel.updateClipInfo(clipManager.clipInfo());
            break;
        case '+':
            timeline.setSpeed(timeline.speed + 0.5);
            break;
        case '-':
            timeline.setSpeed(timeline.speed - 0.5);
            break;
    }
});

// Resize
window.addEventListener('resize', () => {
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    renderer.setSize(width, height);
    godCamera.resize(width, height);
});

// --- Boucle de rendu ---
function animate() {
    requestAnimationFrame(animate);

    // Avancer la timeline (mode replay)
    if (!liveWs) {
        const prevSeason = simState.season;
        const events = timeline.advance();
        for (const event of events) {
            simState.applyEvent(event);

            // Flash d'invasion
            if ((event.event_type || event.e) === 'invade') {
                const data = event.data || event;
                interactionRenderer.flashInvasion(
                    data.x ?? data.cell?.[0],
                    data.y ?? data.cell?.[1],
                    simState.terrainHeights
                );
            }
        }

        if (events.length > 0) {
            if (simState.season !== prevSeason) {
                lighting.setSeason(simState.season);
            }
            plantRenderer.update(simState.getAlivePlants());
            interactionRenderer.updateLinks(simState.links, simState.terrainHeights);
            panel.updateStats(simState);
            panel.updateScrub(timeline.progress());
        }
    }

    lighting.update();
    interactionRenderer.tick();
    renderer.render(scene, camera);
}

// --- Demarrage ---
init();
animate();
