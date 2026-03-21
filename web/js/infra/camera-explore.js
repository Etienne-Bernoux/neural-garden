import * as THREE from 'three';
import { TERRAIN_SCALE } from './terrain.js';

/**
 * Camera d'exploration FPS — balade sur l'ile.
 * Pointer lock + ZQSD/WASD + souris + saut.
 * Taille humaine par rapport aux plantes.
 */
export class ExploreCamera {
    constructor(canvas) {
        this.camera = new THREE.PerspectiveCamera(70, canvas.clientWidth / canvas.clientHeight, 0.1, 500);
        this.camera.position.set(0, 10, 0);

        this.canvas = canvas;
        this.moveSpeed = 0.3;
        this.lookSpeed = 0.002;
        this.heightOffset = 1.7;  // taille humaine (~1.7m)

        // Direction du regard
        this.yaw = 0;
        this.pitch = 0;

        // Touches enfoncees (ZQSD + WASD)
        this.keys = {
            forward: false,   // Z ou W
            left: false,      // Q ou A
            backward: false,  // S
            right: false,     // D
            jump: false,      // Espace
        };

        // Saut
        this.velocityY = 0;
        this.isGrounded = true;
        this.jumpForce = 0.15;
        this.gravity = -0.006;

        // Hauteurs du terrain
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

        // Touches : support ZQSD (AZERTY) + WASD (QWERTY)
        document.addEventListener('keydown', (e) => {
            this._setKey(e.key.toLowerCase(), true);
        });

        document.addEventListener('keyup', (e) => {
            this._setKey(e.key.toLowerCase(), false);
        });
    }

    _setKey(key, pressed) {
        switch (key) {
            case 'z': case 'w': this.keys.forward = pressed; break;
            case 'q': case 'a': this.keys.left = pressed; break;
            case 's': this.keys.backward = pressed; break;
            case 'd': this.keys.right = pressed; break;
            case ' ': this.keys.jump = pressed; break;
        }
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
        if (this.keys.forward) move.add(forward);
        if (this.keys.backward) move.sub(forward);
        if (this.keys.right) move.add(right);
        if (this.keys.left) move.sub(right);

        if (move.length() > 0) {
            move.normalize().multiplyScalar(this.moveSpeed);
            this.camera.position.add(move);
        }

        // Saut
        if (this.keys.jump && this.isGrounded) {
            this.velocityY = this.jumpForce;
            this.isGrounded = false;
        }

        // Gravite
        this.velocityY += this.gravity;

        // Hauteur du terrain
        const terrainY = this._getTerrainHeight(this.camera.position.x, this.camera.position.z);
        const targetY = terrainY + this.heightOffset;

        this.camera.position.y += this.velocityY;

        // Collision avec le sol
        if (this.camera.position.y <= targetY) {
            this.camera.position.y = targetY;
            this.velocityY = 0;
            this.isGrounded = true;
        }

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
        this.velocityY = 0;
        this.isGrounded = true;
    }
}
