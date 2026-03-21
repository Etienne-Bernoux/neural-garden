import * as THREE from 'three';

const SEASON_CONFIG = {
    Spring: { intensity: 0.8, color: 0xffd700, ambient: 0x404040, fog: null },
    Summer: { intensity: 1.0, color: 0xffffff, ambient: 0x505050, fog: null },
    Autumn: { intensity: 0.6, color: 0xff8c00, ambient: 0x353535, fog: null },
    Winter: { intensity: 0.3, color: 0x87ceeb, ambient: 0x252535, fog: { color: 0xccccdd, near: 50, far: 200 } },
};

/**
 * Gestionnaire d'eclairage saisonnier.
 */
export class LightingManager {
    constructor(scene) {
        this.scene = scene;

        // Lumiere directionnelle
        this.dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
        this.dirLight.position.set(50, 80, 50);
        this.dirLight.castShadow = true;
        this.dirLight.shadow.mapSize.width = 1024;
        this.dirLight.shadow.mapSize.height = 1024;
        this.dirLight.shadow.camera.near = 0.5;
        this.dirLight.shadow.camera.far = 300;
        this.dirLight.shadow.camera.left = -100;
        this.dirLight.shadow.camera.right = 100;
        this.dirLight.shadow.camera.top = 100;
        this.dirLight.shadow.camera.bottom = -100;
        scene.add(this.dirLight);

        // Lumiere ambiante
        this.ambientLight = new THREE.AmbientLight(0x404040);
        scene.add(this.ambientLight);

        this.currentSeason = 'Spring';
    }

    /**
     * Met a jour l'eclairage pour la saison donnee.
     */
    setSeason(season) {
        const config = SEASON_CONFIG[season] || SEASON_CONFIG.Spring;
        this.currentSeason = season;

        this.dirLight.intensity = config.intensity;
        this.dirLight.color.set(config.color);
        this.ambientLight.color.set(config.ambient);

        if (config.fog) {
            this.scene.fog = new THREE.Fog(config.fog.color, config.fog.near, config.fog.far);
        } else {
            this.scene.fog = null;
        }
    }
}
