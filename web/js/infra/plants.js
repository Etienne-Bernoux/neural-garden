import * as THREE from 'three';
import { TERRAIN_SCALE } from './terrain.js';

// Couleur de lignée : teinte unique par lineage_id
function lineageHue(lineageId) {
    // Golden ratio hash pour des teintes bien distribuées
    return (lineageId * 0.618033988) % 1.0;
}

function plantColor(lineageId, vitality, maxVitality) {
    const hue = lineageHue(lineageId);
    const healthRatio = maxVitality > 0 ? vitality / maxVitality : 0;
    const saturation = 0.3 + healthRatio * 0.5;
    const lightness = 0.2 + healthRatio * 0.4;
    return new THREE.Color().setHSL(hue, saturation, lightness);
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
                // sinon : pas de changement, on ne reconstruit pas
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
        const color = plantColor(plant.lineage_id, plant.vitality, 100);
        const cells = plant.cells || [];
        const biomass = plant.biomass || cells.length;
        const isMature = plant.state === 'Mature';
        const isSeed = plant.state === 'Seed';
        const isDying = plant.state === 'Dying' || plant.state === 'Dead' || plant.state === 'Decomposing';

        if (isSeed) {
            // Graine : petit cube lumineux
            const geo = new THREE.BoxGeometry(0.4, 0.4, 0.4);
            const mat = new THREE.MeshLambertMaterial({
                color: color.clone().multiplyScalar(0.6),
                emissive: color.clone().multiplyScalar(0.2),
            });
            const mesh = new THREE.Mesh(geo, mat);
            if (cells.length > 0) {
                mesh.position.set(cells[0][0] - this.gridSize/2, this._getHeight(cells[0]) + 0.3, cells[0][1] - this.gridSize/2);
            }
            group.add(mesh);
            return;
        }

        const gridSize = this.gridSize;

        // Tronc : colonne sur la première cellule
        if (cells.length > 0) {
            const trunkHeight = Math.min(1 + Math.floor(biomass / 5), 5);
            const baseHeight = this._getHeight(cells[0]);

            for (let h = 0; h < trunkHeight; h++) {
                const geo = new THREE.BoxGeometry(0.3, 1, 0.3);
                const trunkColor = isDying ? new THREE.Color(0x4a3520) : new THREE.Color(0x5d4e37);
                const mat = new THREE.MeshLambertMaterial({ color: trunkColor });
                const block = new THREE.Mesh(geo, mat);
                block.position.set(
                    cells[0][0] - gridSize/2,
                    baseHeight + h + 0.5,
                    cells[0][1] - gridSize/2
                );
                group.add(block);
            }

            // Canopée : blocs autour du sommet
            const canopyY = baseHeight + trunkHeight;
            const canopySize = Math.max(1, Math.floor(Math.sqrt(biomass)));
            const canopyColor = isDying ? color.clone().multiplyScalar(0.4) : color;

            for (let dx = -canopySize; dx <= canopySize; dx++) {
                for (let dz = -canopySize; dz <= canopySize; dz++) {
                    for (let dy = 0; dy < Math.max(1, canopySize - Math.abs(dx) - Math.abs(dz)); dy++) {
                        if (dx*dx + dz*dz + dy*dy > canopySize*canopySize + 1) continue;

                        const geo = new THREE.BoxGeometry(0.9, 0.9, 0.9);
                        const mat = new THREE.MeshLambertMaterial({ color: canopyColor.clone() });
                        const block = new THREE.Mesh(geo, mat);
                        block.position.set(
                            cells[0][0] - gridSize/2 + dx * 0.9,
                            canopyY + dy * 0.9,
                            cells[0][1] - gridSize/2 + dz * 0.9
                        );
                        group.add(block);
                    }
                }
            }

            // Fleurs si mature
            if (isMature) {
                const flowerGeo = new THREE.BoxGeometry(0.3, 0.3, 0.3);
                const flowerMat = new THREE.MeshLambertMaterial({
                    color: new THREE.Color().setHSL(lineageHue(plant.lineage_id), 0.9, 0.7),
                    emissive: new THREE.Color().setHSL(lineageHue(plant.lineage_id), 0.5, 0.2),
                });
                const flower = new THREE.Mesh(flowerGeo, flowerMat);
                flower.position.set(
                    cells[0][0] - gridSize/2,
                    canopyY + canopySize * 0.9 + 0.3,
                    cells[0][1] - gridSize/2
                );
                group.add(flower);
            }
        }
    }

    _getPlantHash(plant) {
        return `${plant.cells?.length || 0}-${plant.state}-${Math.round((plant.vitality || 0) * 10)}`;
    }

    _getHeight(cell) {
        // Hauteur du terrain — meme echelle que terrain.js
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
