import * as THREE from 'three';
import { TERRAIN_SCALE } from './terrain.js';
import { buildGrass, buildBush, buildTree, buildConifer } from './plant-archetypes.js';

// Couleur de lignée : teinte unique par lineage_id
export function lineageHue(lineageId) {
    // Golden ratio hash pour des teintes bien distribuées
    return (lineageId * 0.618033988) % 1.0;
}

// Couleur de base de la plante selon lignée et vitalité
function plantColor(lineageId, vitality, maxVitality) {
    const hue = lineageHue(lineageId);
    const healthRatio = maxVitality > 0 ? vitality / maxVitality : 0;
    const saturation = 0.3 + healthRatio * 0.5;
    const lightness = 0.2 + healthRatio * 0.4;
    return new THREE.Color().setHSL(hue, saturation, lightness);
}

// Détermine l'archétype depuis max_size
function archetype(maxSize) {
    if (maxSize < 20) return 'grass';
    if (maxSize <= 27) return 'bush';
    if (maxSize <= 35) return 'tree';
    return 'conifer';
}

/**
 * Gestionnaire de rendu des plantes.
 */
export class PlantRenderer {
    constructor(scene, gridSize = 128) {
        this.scene = scene;
        this.plantMeshes = new Map();  // plant_id -> THREE.Group
        this.gridSize = gridSize;
    }

    /**
     * Met à jour le rendu des plantes depuis l'état courant.
     * @param {object[]} plants - Liste des plantes avec id, cells, vitality, biomass, lineage_id, state, traits
     */
    update(plants) {
        const activePlantIds = new Set();

        for (const plant of plants) {
            activePlantIds.add(plant.id);
            const hash = this._getPlantHash(plant);

            if (this.plantMeshes.has(plant.id)) {
                const existing = this.plantMeshes.get(plant.id);
                if (existing.userData.hash !== hash) {
                    this._updatePlant(plant);
                    existing.userData.hash = hash;
                }
            } else {
                this._createPlant(plant);
                this.plantMeshes.get(plant.id).userData.hash = hash;
            }
        }

        // Retirer les plantes qui n'existent plus
        for (const [id, group] of this.plantMeshes) {
            if (!activePlantIds.has(id)) {
                this.scene.remove(group);
                group.traverse(child => {
                    if (child.geometry) child.geometry.dispose();
                    if (child.material) child.material.dispose();
                });
                this.plantMeshes.delete(id);
            }
        }
    }

    _createPlant(plant) {
        const group = new THREE.Group();
        group.userData = { plantId: plant.id };
        this._buildPlantGeometry(group, plant);
        this.scene.add(group);
        this.plantMeshes.set(plant.id, group);
    }

    _updatePlant(plant) {
        const group = this.plantMeshes.get(plant.id);
        // Vider le groupe et reconstruire
        while (group.children.length > 0) {
            const child = group.children[0];
            if (child.geometry) child.geometry.dispose();
            if (child.material) child.material.dispose();
            group.remove(child);
        }
        this._buildPlantGeometry(group, plant);
    }

    _buildPlantGeometry(group, plant) {
        const traits = plant.traits || {};
        const cells = plant.cells || [];
        const biomass = plant.biomass || cells.length;
        const state = plant.state || 'Growing';
        const maxSize = traits.max_size || 25;
        const exudateType = traits.exudate_type || 'carbon';
        const hiddenSize = traits.hidden_size || 8;

        // Position de base (première cellule)
        const cell = cells.length > 0 ? cells[0] : [0, 0];
        const baseX = cell[0] - this.gridSize / 2;
        const baseZ = cell[1] - this.gridSize / 2;
        const baseY = this._getHeight(cell);

        // Les graines sont sous terre — invisibles
        if (state === 'Seed') {
            return;
        }

        // État Decomposing : quasi transparent, gris/brun foncé
        const isDecomposing = state === 'Decomposing';
        const isDying = state === 'Dying' || state === 'Dead';
        const isStressed = state === 'Stressed';
        const isMature = state === 'Mature';

        // Couleur ajustée par état
        let color;
        if (isDecomposing) {
            color = new THREE.Color(0x4a4035);
        } else {
            color = plantColor(plant.lineage_id, plant.vitality, 100);
        }

        // Facteur de taille selon état (Growing = petit, Mature = plein)
        const sizeFactor = state === 'Growing' ? 0.5 : 1.0;
        const effectiveBiomass = biomass * sizeFactor;

        // Paramètres communs pour les constructeurs d'archétype
        const params = {
            biomass: effectiveBiomass,
            isDying: isDying || isDecomposing,
            isStressed,
            isMature,
            id: plant.id,
            exudateType,
            hiddenSize,
        };

        // Teinte de la lignee pour les textures procedurales
        const hue = lineageHue(plant.lineage_id);

        // Dispatch vers le constructeur d'archétype
        const type = archetype(maxSize);
        switch (type) {
            case 'grass':   buildGrass(group, plant, baseX, baseY, baseZ, color, params, hue); break;
            case 'bush':    buildBush(group, plant, baseX, baseY, baseZ, color, params, hue); break;
            case 'tree':    buildTree(group, plant, baseX, baseY, baseZ, color, params, hue); break;
            case 'conifer': buildConifer(group, plant, baseX, baseY, baseZ, color, params, hue); break;
        }

        // Les racines sont sous terre — invisibles

        // Opacité basse pour les plantes en décomposition
        if (isDecomposing) {
            group.traverse(child => {
                if (child.material) {
                    child.material.transparent = true;
                    child.material.opacity = 0.35;
                }
            });
        }
    }

    _getPlantHash(plant) {
        const rootCount = plant.roots ? plant.roots.length : 0;
        return `${plant.cells?.length || 0}-${rootCount}-${plant.state}-${Math.round((plant.vitality || 0) * 10)}-${Math.round((plant.biomass || 0) * 10)}`;
    }

    _getHeight(cell) {
        // Hauteur du terrain — même échelle que terrain.js
        return this._terrainHeights ? (this._terrainHeights[cell[1]]?.[cell[0]] || 0) * TERRAIN_SCALE : 1;
    }

    setTerrainHeights(heights) {
        this._terrainHeights = heights;
    }

    /**
     * Retourne la plante à la position du raycaster
     */
    getPlantAtRaycast(raycaster) {
        for (const [id, group] of this.plantMeshes) {
            const intersects = raycaster.intersectObject(group, true);
            if (intersects.length > 0) {
                return id;
            }
        }
        return null;
    }

    dispose() {
        for (const [, group] of this.plantMeshes) {
            this.scene.remove(group);
            group.traverse(child => {
                if (child.geometry) child.geometry.dispose();
                if (child.material) child.material.dispose();
            });
        }
        this.plantMeshes.clear();
    }
}
