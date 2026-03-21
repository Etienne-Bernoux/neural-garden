import * as THREE from 'three';
import { TERRAIN_SCALE } from './terrain.js';

/**
 * Systeme de particules pour la decomposition des plantes.
 * Quand une plante meurt, ses blocs se fragmentent en particules
 * qui tombent avec gravite et fade progressif.
 */
export class ParticleSystem {
    constructor(scene) {
        this.scene = scene;
        this.emitters = [];  // emissions actives
        this.gridSize = 128;
    }

    /**
     * Declencher une emission de particules a la mort d'une plante.
     * @param {object} plant - la plante morte (cells, lineage_id)
     * @param {number[][]} terrainHeights
     */
    emit(plant, terrainHeights) {
        if (!plant || !plant.cells || plant.cells.length === 0) return;

        const hue = (plant.lineage_id * 0.618033988) % 1.0;
        const color = new THREE.Color().setHSL(hue, 0.4, 0.3);  // couleur ternie

        const particleCount = Math.min(plant.cells.length * 8, 60);  // max 60 particules

        const geometry = new THREE.BufferGeometry();
        const positions = new Float32Array(particleCount * 3);
        const colors = new Float32Array(particleCount * 3);
        const velocities = [];

        // Position initiale : autour des cellules de la plante
        for (let i = 0; i < particleCount; i++) {
            const cellIdx = i % plant.cells.length;
            const cell = plant.cells[cellIdx];
            const alt = terrainHeights?.[cell[1]]?.[cell[0]] || 0;
            const baseY = alt * TERRAIN_SCALE;

            // Position avec jitter aleatoire
            positions[i * 3] = cell[0] - this.gridSize / 2 + (Math.random() - 0.5) * 2;
            positions[i * 3 + 1] = baseY + Math.random() * 4 + 1;  // au-dessus du sol
            positions[i * 3 + 2] = cell[1] - this.gridSize / 2 + (Math.random() - 0.5) * 2;

            // Couleur avec variation
            const variation = 0.8 + Math.random() * 0.4;
            colors[i * 3] = color.r * variation;
            colors[i * 3 + 1] = color.g * variation;
            colors[i * 3 + 2] = color.b * variation;

            // Velocite : explosion vers l'exterieur puis gravite
            velocities.push({
                x: (Math.random() - 0.5) * 0.15,
                y: Math.random() * 0.2 + 0.05,  // leger ejection vers le haut
                z: (Math.random() - 0.5) * 0.15,
                groundY: baseY,  // hauteur du sol pour l'arret
            });
        }

        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

        const material = new THREE.PointsMaterial({
            size: 0.4,
            vertexColors: true,
            transparent: true,
            opacity: 1.0,
            sizeAttenuation: true,
        });

        const points = new THREE.Points(geometry, material);
        this.scene.add(points);

        this.emitters.push({
            points,
            velocities,
            age: 0,
            maxAge: 90,  // ~3 secondes a 30fps
        });
    }

    /**
     * Mettre a jour les particules (appeler chaque frame).
     */
    update() {
        const gravity = -0.008;
        const friction = 0.98;

        for (let i = this.emitters.length - 1; i >= 0; i--) {
            const emitter = this.emitters[i];
            emitter.age++;

            // Fade out
            const lifeRatio = emitter.age / emitter.maxAge;
            emitter.points.material.opacity = Math.max(0, 1.0 - lifeRatio);

            // Mettre a jour les positions
            const positions = emitter.points.geometry.attributes.position;

            for (let j = 0; j < emitter.velocities.length; j++) {
                const vel = emitter.velocities[j];

                // Appliquer la gravite
                vel.y += gravity;

                // Friction
                vel.x *= friction;
                vel.z *= friction;

                // Deplacer
                positions.array[j * 3] += vel.x;
                positions.array[j * 3 + 1] += vel.y;
                positions.array[j * 3 + 2] += vel.z;

                // Collision sol : rebondir faiblement puis s'arreter
                if (positions.array[j * 3 + 1] < vel.groundY + 0.1) {
                    positions.array[j * 3 + 1] = vel.groundY + 0.1;
                    vel.y = Math.abs(vel.y) * 0.2;  // rebond amorti
                    vel.x *= 0.5;
                    vel.z *= 0.5;
                }
            }

            positions.needsUpdate = true;

            // Retirer si trop vieux
            if (emitter.age >= emitter.maxAge) {
                this.scene.remove(emitter.points);
                emitter.points.geometry.dispose();
                emitter.points.material.dispose();
                this.emitters.splice(i, 1);
            }
        }
    }

    setGridSize(size) {
        this.gridSize = size;
    }

    dispose() {
        for (const emitter of this.emitters) {
            this.scene.remove(emitter.points);
            emitter.points.geometry.dispose();
            emitter.points.material.dispose();
        }
        this.emitters.length = 0;
    }
}
