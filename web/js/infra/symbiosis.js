import * as THREE from 'three';
import { TERRAIN_SCALE } from './terrain.js';

/**
 * Gestionnaire de rendu des interactions (symbiose, exsudats, invasion).
 */
export class InteractionRenderer {
    constructor(scene, gridSize = 128) {
        this.scene = scene;
        this.linkMeshes = [];
        this.exudateMeshes = [];
        this.flashMeshes = [];
        this.gridSize = gridSize;
    }

    /**
     * Met à jour les liens mycorhiziens.
     * @param {object[]} links - [{plant_a, plant_b, pos_a, pos_b}]
     */
    updateLinks(links, terrainHeights) {
        // Nettoyer les anciens liens
        this._clearGroup(this.linkMeshes);

        for (const link of links) {
            const posA = link.pos_a;
            const posB = link.pos_b;
            if (!posA || !posB) continue;

            const heightA = (terrainHeights?.[posA[1]]?.[posA[0]] || 0) * TERRAIN_SCALE + 0.2;
            const heightB = (terrainHeights?.[posB[1]]?.[posB[0]] || 0) * TERRAIN_SCALE + 0.2;

            const start = new THREE.Vector3(posA[0] - this.gridSize/2, heightA, posA[1] - this.gridSize/2);
            const end = new THREE.Vector3(posB[0] - this.gridSize/2, heightB, posB[1] - this.gridSize/2);

            // Filament doré
            const curve = new THREE.CatmullRomCurve3([
                start,
                new THREE.Vector3((start.x + end.x) / 2, Math.min(heightA, heightB) - 0.3, (start.z + end.z) / 2),
                end
            ]);
            const tubeGeo = new THREE.TubeGeometry(curve, 8, 0.05, 4, false);
            const tubeMat = new THREE.MeshBasicMaterial({
                color: 0xd4a017,
                transparent: true,
                opacity: 0.7,
            });
            const tube = new THREE.Mesh(tubeGeo, tubeMat);
            this.scene.add(tube);
            this.linkMeshes.push(tube);
        }
    }

    /**
     * Met à jour les halos d'exsudats.
     * @param {object[]} exudates - [{x, y, type, intensity}]
     */
    updateExudates(exudates, terrainHeights) {
        this._clearGroup(this.exudateMeshes);

        for (const ex of exudates) {
            const height = (terrainHeights?.[ex.y]?.[ex.x] || 0) * TERRAIN_SCALE + 0.1;
            const color = ex.type === 'carbon' ? 0xd4a017 : 0x3498db;
            const radius = 0.5 + ex.intensity * 2;

            const geo = new THREE.CircleGeometry(radius, 16);
            const mat = new THREE.MeshBasicMaterial({
                color,
                transparent: true,
                opacity: 0.2 + ex.intensity * 0.3,
                side: THREE.DoubleSide,
            });
            const mesh = new THREE.Mesh(geo, mat);
            mesh.rotation.x = -Math.PI / 2;
            mesh.position.set(ex.x - this.gridSize/2, height, ex.y - this.gridSize/2);
            this.scene.add(mesh);
            this.exudateMeshes.push(mesh);
        }
    }

    /**
     * Flash d'invasion (rouge bref).
     * @param {number} x
     * @param {number} y
     */
    flashInvasion(x, y, terrainHeights) {
        const height = (terrainHeights?.[y]?.[x] || 0) * TERRAIN_SCALE + 1;
        const geo = new THREE.BoxGeometry(1, 2, 1);
        const mat = new THREE.MeshBasicMaterial({
            color: 0xff0000,
            transparent: true,
            opacity: 0.6,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.position.set(x - this.gridSize/2, height, y - this.gridSize/2);
        this.scene.add(mesh);
        this.flashMeshes.push({ mesh, ttl: 10 });  // 10 frames
    }

    /**
     * Tick d'animation : faire disparaître les flashs.
     */
    tick() {
        for (let i = this.flashMeshes.length - 1; i >= 0; i--) {
            this.flashMeshes[i].ttl--;
            this.flashMeshes[i].mesh.material.opacity *= 0.85;
            if (this.flashMeshes[i].ttl <= 0) {
                this.scene.remove(this.flashMeshes[i].mesh);
                this.flashMeshes[i].mesh.geometry.dispose();
                this.flashMeshes[i].mesh.material.dispose();
                this.flashMeshes.splice(i, 1);
            }
        }
    }

    _clearGroup(arr) {
        for (const mesh of arr) {
            this.scene.remove(mesh);
            if (mesh.geometry) mesh.geometry.dispose();
            if (mesh.material) mesh.material.dispose();
        }
        arr.length = 0;
    }

    dispose() {
        this._clearGroup(this.linkMeshes);
        this._clearGroup(this.exudateMeshes);
        for (const f of this.flashMeshes) {
            this.scene.remove(f.mesh);
            f.mesh.geometry.dispose();
            f.mesh.material.dispose();
        }
        this.flashMeshes.length = 0;
    }
}
