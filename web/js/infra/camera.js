import * as THREE from 'three';

/**
 * Camera isometrique (mode Dieu).
 */
export class GodCamera {
    constructor(canvas) {
        const aspect = canvas.clientWidth / canvas.clientHeight;
        const frustum = 50;  // plus proche de l'ile
        this.camera = new THREE.OrthographicCamera(
            -frustum * aspect, frustum * aspect,
            frustum, -frustum,
            0.1, 1000
        );

        // Position iso classique, plus basse
        this.camera.position.set(80, 80, 80);
        this.camera.lookAt(0, 3, 0);

        this.canvas = canvas;
        this.isDragging = false;
        this.lastMouse = { x: 0, y: 0 };
        this.target = new THREE.Vector3(0, 0, 0);

        this._setupControls();
    }

    _setupControls() {
        this.canvas.addEventListener('mousedown', (e) => {
            this.isDragging = true;
            this.lastMouse = { x: e.clientX, y: e.clientY };
        });

        window.addEventListener('mouseup', () => {
            this.isDragging = false;
        });

        window.addEventListener('mousemove', (e) => {
            if (!this.isDragging) return;
            const dx = e.clientX - this.lastMouse.x;
            const dy = e.clientY - this.lastMouse.y;
            this.lastMouse = { x: e.clientX, y: e.clientY };

            // Pan
            this.target.x -= dx * 0.3;
            this.target.z -= dy * 0.3;
            this._updatePosition();
        });

        this.canvas.addEventListener('wheel', (e) => {
            e.preventDefault();
            // Zoom
            const zoomFactor = e.deltaY > 0 ? 1.1 : 0.9;
            this.camera.left *= zoomFactor;
            this.camera.right *= zoomFactor;
            this.camera.top *= zoomFactor;
            this.camera.bottom *= zoomFactor;
            this.camera.updateProjectionMatrix();
        }, { passive: false });
    }

    _updatePosition() {
        this.camera.position.set(
            this.target.x + 80,
            80,
            this.target.z + 80
        );
        this.camera.lookAt(this.target);
    }

    resize(width, height) {
        const aspect = width / height;
        const frustum = Math.abs(this.camera.top);
        this.camera.left = -frustum * aspect;
        this.camera.right = frustum * aspect;
        this.camera.updateProjectionMatrix();
    }

    getCamera() { return this.camera; }
}
