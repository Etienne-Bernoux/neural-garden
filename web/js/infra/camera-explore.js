import * as THREE from 'three';
import { TERRAIN_SCALE } from './terrain.js';

/**
 * Camera d'exploration FPS — balade sur l'ile.
 * Pointer lock + WASD + souris.
 */
export class ExploreCamera {
    constructor(canvas) {
        this.camera = new THREE.PerspectiveCamera(70, canvas.clientWidth / canvas.clientHeight, 0.1, 500);
        this.camera.position.set(0, 10, 0);

        this.canvas = canvas;
        this.moveSpeed = 0.5;
        this.lookSpeed = 0.002;
        this.heightOffset = 3;  // hauteur au-dessus du terrain

        // Direction du regard
        this.yaw = 0;    // rotation horizontale
        this.pitch = 0;  // rotation verticale (clampee)

        // Touches enfoncees
        this.keys = { w: false, a: false, s: false, d: false };

        // Hauteurs du terrain (injectees)
        this._terrainHeights = null;
        this._gridSize = 128;

        this.locked = false;

        this._setupControls();
    }

    _setupControls() {
        // Pointer lock au clic
        this.canvas.addEventListener('click', () => {
            if (!this.locked) {
                this.canvas.requestPointerLock();
            }
        });

        document.addEventListener('pointerlockchange', () => {
            this.locked = document.pointerLockElement === this.canvas;
        });

        // Mouvement souris
        document.addEventListener('mousemove', (e) => {
            if (!this.locked) return;
            this.yaw -= e.movementX * this.lookSpeed;
            this.pitch -= e.movementY * this.lookSpeed;
            this.pitch = Math.max(-Math.PI / 2 + 0.1, Math.min(Math.PI / 2 - 0.1, this.pitch));
        });

        // Touches
        document.addEventListener('keydown', (e) => {
            const key = e.key.toLowerCase();
            if (key in this.keys) this.keys[key] = true;
        });

        document.addEventListener('keyup', (e) => {
            const key = e.key.toLowerCase();
            if (key in this.keys) this.keys[key] = false;
        });
    }

    /**
     * Met a jour la position de la camera (appeler chaque frame).
     */
    update() {
        if (!this.locked) return;

        // Direction de deplacement basee sur le yaw
        const forward = new THREE.Vector3(
            -Math.sin(this.yaw),
            0,
            -Math.cos(this.yaw)
        );
        const right = new THREE.Vector3(
            Math.cos(this.yaw),
            0,
            -Math.sin(this.yaw)
        );

        const move = new THREE.Vector3();
        if (this.keys.w) move.add(forward);
        if (this.keys.s) move.sub(forward);
        if (this.keys.d) move.add(right);
        if (this.keys.a) move.sub(right);

        if (move.length() > 0) {
            move.normalize().multiplyScalar(this.moveSpeed);
            this.camera.position.add(move);
        }

        // Suivre le relief
        const terrainY = this._getTerrainHeight(this.camera.position.x, this.camera.position.z);
        this.camera.position.y = terrainY + this.heightOffset;

        // Orienter la camera
        const lookDir = new THREE.Vector3(
            -Math.sin(this.yaw) * Math.cos(this.pitch),
            Math.sin(this.pitch),
            -Math.cos(this.yaw) * Math.cos(this.pitch)
        );
        const target = this.camera.position.clone().add(lookDir);
        this.camera.lookAt(target);
    }

    /**
     * Hauteur du terrain a une position world.
     */
    _getTerrainHeight(wx, wz) {
        if (!this._terrainHeights) return 0;

        // Convertir world coords en grid coords
        const gx = Math.round(wx + this._gridSize / 2);
        const gz = Math.round(wz + this._gridSize / 2);

        if (gx < 0 || gx >= this._gridSize || gz < 0 || gz >= this._gridSize) return 0;

        const alt = this._terrainHeights[gz]?.[gx] || 0;
        return alt * TERRAIN_SCALE;
    }

    setTerrainHeights(heights, gridSize) {
        this._terrainHeights = heights;
        this._gridSize = gridSize || 128;
    }

    resize(width, height) {
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
    }

    getCamera() { return this.camera; }

    /**
     * Teleporter la camera a une position.
     */
    teleportTo(x, z) {
        this.camera.position.x = x;
        this.camera.position.z = z;
        const terrainY = this._getTerrainHeight(x, z);
        this.camera.position.y = terrainY + this.heightOffset;
    }
}
