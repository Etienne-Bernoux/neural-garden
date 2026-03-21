import * as THREE from 'three';

// Cache de textures — une seule instance par cle
const textureCache = new Map();

/**
 * Genere une texture procedurale via Canvas2D et la met en cache.
 */
function generateTexture(key, width, height, drawFn) {
    if (textureCache.has(key)) return textureCache.get(key);

    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    drawFn(ctx, width, height);

    const texture = new THREE.CanvasTexture(canvas);
    texture.wrapS = THREE.RepeatWrapping;
    texture.wrapT = THREE.RepeatWrapping;
    textureCache.set(key, texture);
    return texture;
}

// --- Textures du sol ---

/**
 * Sol herbeux (humidite haute, altitude moyenne)
 */
export function grassTexture() {
    return generateTexture('grass', 64, 64, (ctx, w, h) => {
        // Fond vert
        ctx.fillStyle = '#3a7d2c';
        ctx.fillRect(0, 0, w, h);
        // Brins d'herbe aleatoires
        for (let i = 0; i < 80; i++) {
            const x = Math.random() * w;
            const y = Math.random() * h;
            const len = 3 + Math.random() * 8;
            const hue = 90 + Math.random() * 40;
            ctx.strokeStyle = `hsl(${hue}, 60%, ${25 + Math.random() * 20}%)`;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(x, y);
            ctx.lineTo(x + (Math.random() - 0.5) * 3, y - len);
            ctx.stroke();
        }
    });
}

/**
 * Sol sec/sablonneux (humidite basse)
 */
export function dryTexture() {
    return generateTexture('dry', 64, 64, (ctx, w, h) => {
        ctx.fillStyle = '#b8a070';
        ctx.fillRect(0, 0, w, h);
        // Grains de sable
        for (let i = 0; i < 100; i++) {
            const x = Math.random() * w;
            const y = Math.random() * h;
            ctx.fillStyle = `hsl(35, ${30 + Math.random() * 20}%, ${55 + Math.random() * 15}%)`;
            ctx.fillRect(x, y, 1 + Math.random() * 2, 1 + Math.random() * 2);
        }
    });
}

/**
 * Sol forestier dense (beaucoup de biomasse)
 */
export function forestFloorTexture() {
    return generateTexture('forest', 64, 64, (ctx, w, h) => {
        ctx.fillStyle = '#2d5a1e';
        ctx.fillRect(0, 0, w, h);
        // Mousse et feuilles mortes
        for (let i = 0; i < 60; i++) {
            const x = Math.random() * w;
            const y = Math.random() * h;
            const hue = 70 + Math.random() * 60;
            ctx.fillStyle = `hsl(${hue}, ${40 + Math.random() * 30}%, ${15 + Math.random() * 20}%)`;
            ctx.beginPath();
            ctx.arc(x, y, 1 + Math.random() * 3, 0, Math.PI * 2);
            ctx.fill();
        }
    });
}

/**
 * Sol rocheux (altitude haute)
 */
export function rockTexture() {
    return generateTexture('rock', 64, 64, (ctx, w, h) => {
        ctx.fillStyle = '#6b6155';
        ctx.fillRect(0, 0, w, h);
        // Fissures et variations
        for (let i = 0; i < 40; i++) {
            const x = Math.random() * w;
            const y = Math.random() * h;
            ctx.fillStyle = `hsl(30, ${10 + Math.random() * 15}%, ${30 + Math.random() * 20}%)`;
            ctx.fillRect(x, y, 2 + Math.random() * 5, 1 + Math.random() * 3);
        }
    });
}

/**
 * Sol cotier (altitude basse, pres de la mer)
 */
export function sandTexture() {
    return generateTexture('sand', 64, 64, (ctx, w, h) => {
        ctx.fillStyle = '#d4c090';
        ctx.fillRect(0, 0, w, h);
        for (let i = 0; i < 80; i++) {
            const x = Math.random() * w;
            const y = Math.random() * h;
            ctx.fillStyle = `hsl(40, ${30 + Math.random() * 20}%, ${65 + Math.random() * 15}%)`;
            ctx.fillRect(x, y, 1, 1);
        }
    });
}

// --- Texture eau ---

export function waterTexture() {
    return generateTexture('water', 128, 128, (ctx, w, h) => {
        // Fond bleu profond
        ctx.fillStyle = '#1a5276';
        ctx.fillRect(0, 0, w, h);
        // Ondulations
        for (let y = 0; y < h; y += 3) {
            ctx.strokeStyle = `rgba(100, 180, 220, ${0.05 + Math.random() * 0.1})`;
            ctx.lineWidth = 1;
            ctx.beginPath();
            for (let x = 0; x < w; x++) {
                ctx.lineTo(x, y + Math.sin(x * 0.2 + y * 0.1) * 2);
            }
            ctx.stroke();
        }
    });
}

// --- Textures plantes ---

/**
 * Texture ecorce (tronc)
 */
export function barkTexture() {
    return generateTexture('bark', 32, 64, (ctx, w, h) => {
        ctx.fillStyle = '#5a4030';
        ctx.fillRect(0, 0, w, h);
        // Lignes verticales d'ecorce
        for (let i = 0; i < 15; i++) {
            const x = Math.random() * w;
            ctx.strokeStyle = `hsl(25, ${20 + Math.random() * 20}%, ${20 + Math.random() * 15}%)`;
            ctx.lineWidth = 1 + Math.random() * 2;
            ctx.beginPath();
            ctx.moveTo(x, 0);
            ctx.lineTo(x + (Math.random() - 0.5) * 4, h);
            ctx.stroke();
        }
    });
}

/**
 * Texture feuillage (canopee) — parametree par la teinte de la lignee
 */
export function foliageTexture(hue) {
    const key = `foliage_${Math.round(hue * 100)}`;
    return generateTexture(key, 32, 32, (ctx, w, h) => {
        // Fond couleur de la lignee
        const baseH = hue * 360;
        ctx.fillStyle = `hsl(${baseH}, 50%, 35%)`;
        ctx.fillRect(0, 0, w, h);
        // Feuilles
        for (let i = 0; i < 30; i++) {
            const x = Math.random() * w;
            const y = Math.random() * h;
            const leafHue = baseH + (Math.random() - 0.5) * 30;
            ctx.fillStyle = `hsl(${leafHue}, ${40 + Math.random() * 30}%, ${25 + Math.random() * 25}%)`;
            ctx.beginPath();
            ctx.ellipse(x, y, 2 + Math.random() * 3, 1 + Math.random() * 2, Math.random() * Math.PI, 0, Math.PI * 2);
            ctx.fill();
        }
    });
}

/**
 * Texture herbe/touffe — parametree par la teinte de la lignee
 */
export function tuffTexture(hue) {
    const key = `tuff_${Math.round(hue * 100)}`;
    return generateTexture(key, 32, 32, (ctx, w, h) => {
        ctx.fillStyle = 'transparent';
        ctx.clearRect(0, 0, w, h);
        const baseH = hue * 360;
        for (let i = 0; i < 20; i++) {
            const x = Math.random() * w;
            const leafHue = baseH + (Math.random() - 0.5) * 30;
            ctx.strokeStyle = `hsl(${leafHue}, 50%, ${30 + Math.random() * 20}%)`;
            ctx.lineWidth = 1 + Math.random();
            ctx.beginPath();
            ctx.moveTo(x, h);
            ctx.lineTo(x + (Math.random() - 0.5) * 6, h * 0.2 + Math.random() * h * 0.3);
            ctx.stroke();
        }
    });
}

// --- Selection de texture de sol par biome ---

/**
 * Retourne la texture de sol appropriee selon les conditions.
 * @param {number} altitude - 0.0-1.0
 * @param {number} humidity - 0.0-1.0
 */
export function biomeTexture(altitude, humidity) {
    if (altitude > 0.7) return rockTexture();
    if (altitude < 0.25) return sandTexture();
    if (humidity > 0.6) return forestFloorTexture();
    if (humidity < 0.3) return dryTexture();
    return grassTexture();
}
