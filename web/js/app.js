import * as THREE from 'three';
import { createTerrain } from './infra/terrain.js';
import { PlantRenderer } from './infra/plants.js';
import { InteractionRenderer } from './infra/symbiosis.js';
import { SimState } from './domain/state.js';
import { ClipManager } from './domain/clips.js';
import { Timeline } from './application/timeline.js';
import { GodCamera } from './infra/camera.js';
import { ExploreCamera } from './infra/camera-explore.js';
import { LightingManager } from './infra/lighting.js';
import { ParticleSystem } from './infra/particles.js';
import { PanelManager } from './ui/panel.js';
import { BrainViz } from './ui/brain-viz.js';

// --- Init ---
const canvas = document.getElementById('canvas');
const scene = new THREE.Scene();
scene.background = new THREE.Color(0x1a6b8a);  // meme couleur que l'eau

const godCamera = new GodCamera(canvas);
const exploreCamera = new ExploreCamera(canvas);
let currentCameraMode = 'god';  // 'god' ou 'explore'

const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setSize(canvas.clientWidth, canvas.clientHeight, false);
renderer.shadowMap.enabled = true;

const lighting = new LightingManager(scene);
lighting.setCameraMode('god');  // cacher soleil/nuages en mode aerien par defaut
const simState = new SimState();
const plantRenderer = new PlantRenderer(scene, simState.gridSize);
const interactionRenderer = new InteractionRenderer(scene, simState.gridSize);
const particleSystem = new ParticleSystem(scene);
const panel = new PanelManager();
const brainViz = new BrainViz();
const clipManager = new ClipManager();
const timeline = new Timeline();

let terrainGroup = null;
let selectedPlantId = null;

// Controle de vitesse d'affichage (throttle cote viewer)
let displaySpeed = 1;
let tickCounter = 0;

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

    // Injecter les hauteurs du terrain dans les renderers
    plantRenderer.setTerrainHeights(simState.terrainHeights);
    exploreCamera.setTerrainHeights(simState.terrainHeights, simState.gridSize);

    // Injecter gridSize dans le systeme de particules
    particleSystem.setGridSize(simState.gridSize);

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
                exploreCamera.setTerrainHeights(simState.terrainHeights, simState.gridSize);
                particleSystem.setGridSize(simState.gridSize);
            } else if (data.type === 'tick') {
                // Throttle : ne traiter qu'un tick sur N selon la vitesse
                tickCounter++;
                if (tickCounter % Math.max(1, Math.round(1 / displaySpeed)) !== 0) return;

                // Events incrementaux
                if (data.events) {
                    for (const e of data.events) {
                        // Particules de decomposition a la mort
                        const eType = (e.event_type || e.e || '').toLowerCase();
                        if (eType === 'died') {
                            const evtData = e.data || e;
                            const plantId = evtData.plant_id || evtData.p;
                            const plant = simState.plants.get(plantId);
                            if (plant) {
                                particleSystem.emit(plant, simState.terrainHeights);
                            }
                        }
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

// Controles de vitesse d'affichage
document.getElementById('btn-faster')?.addEventListener('click', () => {
    displaySpeed = Math.min(displaySpeed * 2, 8);
    document.getElementById('speed-display').textContent = displaySpeed + 'x';
});
document.getElementById('btn-slower')?.addEventListener('click', () => {
    displaySpeed = Math.max(displaySpeed / 2, 0.25);
    document.getElementById('speed-display').textContent = displaySpeed + 'x';
});

// Selection de plante par clic
const raycaster = new THREE.Raycaster();
const mouse = new THREE.Vector2();

canvas.addEventListener('click', (e) => {
    // Pas de selection en mode explore (le clic sert au pointer lock)
    if (currentCameraMode === 'explore') return;

    const rect = canvas.getBoundingClientRect();
    mouse.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
    mouse.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;

    const activeCamera = currentCameraMode === 'god' ? godCamera.getCamera() : exploreCamera.getCamera();
    raycaster.setFromCamera(mouse, activeCamera);
    selectedPlantId = plantRenderer.getPlantAtRaycast(raycaster);

    if (selectedPlantId) {
        const plant = simState.plants.get(selectedPlantId);
        panel.selectPlant(plant || null);
        brainViz.draw(plant, null, null);  // inputs/outputs pas dispo dans le viewer
    } else {
        panel.selectPlant(null);
        brainViz.draw(null, null, null);
    }
});

// Clavier
document.addEventListener('keydown', (e) => {
    switch (e.key) {
        case ' ':
            e.preventDefault();
            // En mode explore, espace = saut (gere par camera-explore)
            if (currentCameraMode !== 'explore') {
                timeline.togglePlay();
            }
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
        case 'b':
            brainViz.toggle();
            // Redessiner avec la plante sélectionnée
            if (selectedPlantId) {
                const plant = simState.plants.get(selectedPlantId);
                brainViz.draw(plant, null, null);
            }
            break;
        case 'v':
            if (currentCameraMode === 'god') {
                currentCameraMode = 'explore';
                exploreCamera.enabled = true;
                exploreCamera.setTerrainHeights(simState.terrainHeights, simState.gridSize);
                exploreCamera.teleportTo(0, 0);
            } else {
                currentCameraMode = 'god';
                exploreCamera.enabled = false;
                document.exitPointerLock();
            }
            lighting.setCameraMode(currentCameraMode);
            break;
    }
});

// Resize
window.addEventListener('resize', () => {
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    renderer.setSize(width, height, false);
    godCamera.resize(width, height);
    exploreCamera.resize(width, height);
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

            // Particules de decomposition a la mort
            const replayType = (event.event_type || event.e || '').toLowerCase();
            if (replayType === 'died') {
                const data = event.data || event;
                const plantId = data.plant_id || data.p;
                const plant = simState.plants.get(plantId);
                if (plant) {
                    particleSystem.emit(plant, simState.terrainHeights);
                }
            }

            // Flash d'invasion
            if (replayType === 'invaded' || replayType === 'invade') {
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
    particleSystem.update();

    // Camera active selon le mode
    const activeCamera = currentCameraMode === 'god' ? godCamera.getCamera() : exploreCamera.getCamera();
    if (currentCameraMode === 'explore') {
        exploreCamera.update();
    }
    renderer.render(scene, activeCamera);
}

// --- Demarrage ---
init();
animate();
