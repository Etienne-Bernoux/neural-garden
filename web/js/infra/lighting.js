import * as THREE from 'three';

const SEASON_CONFIG = {
    Spring: { intensity: 1.0, color: 0xfff5e0, ambient: 0x606060, fog: null },
    Summer: { intensity: 1.2, color: 0xffffff, ambient: 0x707070, fog: null },
    Autumn: { intensity: 0.9, color: 0xffe0c0, ambient: 0x505050, fog: null },
    Winter: { intensity: 0.6, color: 0xe0e8ff, ambient: 0x404050, fog: { color: 0xccccdd, near: 100, far: 300 } },
};

/**
 * Gestionnaire d'eclairage saisonnier avec interpolation douce.
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

        // Cibles pour l'interpolation
        this.targetIntensity = 1.0;
        this.targetColor = new THREE.Color(0xfff5e0);
        this.targetAmbient = new THREE.Color(0x606060);
        this.targetFogColor = null;
        this.targetFogNear = 0;
        this.targetFogFar = 0;
        this.lerpSpeed = 0.02;  // 2% par frame — transition ~2-3 secondes
    }

    /**
     * Definit la saison cible. L'interpolation se fait dans update().
     */
    setSeason(season) {
        const config = SEASON_CONFIG[season] || SEASON_CONFIG.Spring;
        this.currentSeason = season;

        // Stocker les cibles au lieu d'appliquer directement
        this.targetIntensity = config.intensity;
        this.targetColor = new THREE.Color(config.color);
        this.targetAmbient = new THREE.Color(config.ambient);
        this.targetFogColor = config.fog ? new THREE.Color(config.fog.color) : null;
        this.targetFogNear = config.fog?.near || 0;
        this.targetFogFar = config.fog?.far || 0;
    }

    /**
     * Appeler a chaque frame pour interpoler l'eclairage.
     */
    update() {
        const s = this.lerpSpeed;

        // Interpoler l'intensite
        this.dirLight.intensity += (this.targetIntensity - this.dirLight.intensity) * s;

        // Interpoler les couleurs
        this.dirLight.color.lerp(this.targetColor, s);
        this.ambientLight.color.lerp(this.targetAmbient, s);

        // Interpoler le brouillard
        if (this.targetFogColor) {
            if (!this.scene.fog) {
                this.scene.fog = new THREE.Fog(this.targetFogColor, 300, 500);
            }
            this.scene.fog.color.lerp(this.targetFogColor, s);
            this.scene.fog.near += (this.targetFogNear - this.scene.fog.near) * s;
            this.scene.fog.far += (this.targetFogFar - this.scene.fog.far) * s;
        } else if (this.scene.fog) {
            // Dissiper le brouillard progressivement
            this.scene.fog.near += (500 - this.scene.fog.near) * s;
            this.scene.fog.far += (1000 - this.scene.fog.far) * s;
            if (this.scene.fog.near > 450) {
                this.scene.fog = null;
            }
        }
    }
}
