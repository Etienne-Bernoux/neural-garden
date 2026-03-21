import * as THREE from 'three';
import { barkTexture, foliageTexture, tuffTexture } from './textures.js';

// Pseudo-random déterministe basé sur l'id de la plante
function pseudoRandom(seed) {
    let x = Math.sin(seed * 12.9898) * 43758.5453;
    return x - Math.floor(x);
}

// Séquence pseudo-random : chaque appel donne une valeur différente
function prng(id, index) {
    return pseudoRandom(id * 137 + index * 31);
}

// Couleur du tronc selon l'état
function trunkColor(isDying) {
    return isDying ? new THREE.Color(0x4a3520) : new THREE.Color(0x5d4e37);
}

// Crée un bloc (mesh) positionné
function block(geo, mat, x, y, z) {
    const mesh = new THREE.Mesh(geo, mat);
    mesh.position.set(x, y, z);
    return mesh;
}

/**
 * Herbe : basse, large, pas de tronc visible.
 * Plusieurs touffes de blocs au sol (2-4 clusters).
 * Mature : petites fleurs colorées.
 */
export function buildGrass(group, plant, baseX, baseY, baseZ, color, params, hue) {
    const { biomass, isDying, isStressed, isMature, id } = params;
    const grassColor = isDying
        ? new THREE.Color(0x8b7d3a)
        : isStressed
            ? color.clone().lerp(new THREE.Color(0xb0a030), 0.5)
            : color;

    // Texture de touffe parametree par la lignee
    const grassTex = hue !== undefined ? tuffTexture(hue) : null;

    // Nombre de touffes : 2-4 selon biomasse
    const clusterCount = Math.min(2 + Math.floor(biomass / 8), 4);
    const blockGeo = new THREE.BoxGeometry(1.0, 0.7, 1.0);
    let blockIdx = 0;

    for (let c = 0; c < clusterCount; c++) {
        const ox = (prng(id, c * 2) - 0.5) * 2.0;
        const oz = (prng(id, c * 2 + 1) - 0.5) * 2.0;
        // Chaque touffe : 1-3 blocs empilés (plus hauts)
        const height = Math.max(1, Math.floor(biomass / 8));
        for (let h = 0; h < Math.min(height, 3); h++) {
            const variation = prng(id, blockIdx + 10) * 0.15;
            const matOpts = { color: grassColor.clone().offsetHSL(0, 0, variation - 0.07) };
            if (grassTex) matOpts.map = grassTex;
            const mat = new THREE.MeshLambertMaterial(matOpts);
            group.add(block(blockGeo, mat, baseX + ox, baseY + h * 0.8 + 0.35, baseZ + oz));
            blockIdx++;
        }
    }

    // Fleurs si mature
    if (isMature) {
        const flowerGeo = new THREE.BoxGeometry(0.25, 0.25, 0.25);
        for (let f = 0; f < Math.min(clusterCount, 3); f++) {
            const fx = (prng(id, f * 2 + 50) - 0.5) * 1.5;
            const fz = (prng(id, f * 2 + 51) - 0.5) * 1.5;
            const hue = prng(id, f + 60);
            const flowerMat = new THREE.MeshLambertMaterial({
                color: new THREE.Color().setHSL(hue, 0.9, 0.65),
                emissive: new THREE.Color().setHSL(hue, 0.5, 0.15),
            });
            group.add(block(flowerGeo, flowerMat, baseX + fx, baseY + 1.4, baseZ + fz));
        }
    }
}

/**
 * Buisson : forme sphérique étalée, tronc court (~1 bloc).
 * exudate=Nitrogen → plus étalé, exudate=Carbon → plus compact.
 */
export function buildBush(group, plant, baseX, baseY, baseZ, color, params, hue) {
    const { biomass, isDying, isStressed, isMature, id, exudateType, hiddenSize } = params;
    const canopyColor = isDying
        ? color.clone().multiplyScalar(0.35)
        : isStressed
            ? color.clone().lerp(new THREE.Color(0x808060), 0.4)
            : color;

    // Tronc court avec texture d'ecorce
    const trunkGeo = new THREE.BoxGeometry(0.5, 1.5, 0.5);
    const barkTex = barkTexture();
    const trunkMat = new THREE.MeshLambertMaterial({ map: barkTex, color: trunkColor(isDying) });
    group.add(block(trunkGeo, trunkMat, baseX, baseY + 0.75, baseZ));

    // Texture de feuillage parametree par la lignee
    const foliageTex = hue !== undefined ? foliageTexture(hue) : null;

    // Canopée sphérique (spheres basse resolution = organique)
    const spread = exudateType === 'nitrogen' ? 1.6 : 1.2;
    const verticalScale = exudateType === 'nitrogen' ? 0.7 : 1.0;
    const radius = Math.max(1.2, Math.min(Math.sqrt(biomass) * 0.8, 3.0));
    const canopyBase = baseY + 1.5;
    const blockGeo = new THREE.SphereGeometry(0.6, 6, 6);
    let count = 0;

    for (let dx = -2; dx <= 2; dx++) {
        for (let dz = -2; dz <= 2; dz++) {
            for (let dy = 0; dy <= 1; dy++) {
                const dist = (dx * dx) / (spread * spread) + (dz * dz) / (spread * spread) + (dy * dy) / (verticalScale * verticalScale);
                if (dist > radius * radius) continue;
                if (count >= 20) break;

                const jx = (prng(id, count * 3) - 0.5) * 0.2;
                const jz = (prng(id, count * 3 + 1) - 0.5) * 0.2;
                const variation = prng(id, count * 3 + 2) * 0.1;
                const matOpts = { color: canopyColor.clone().offsetHSL(0, 0, variation - 0.05) };
                if (foliageTex) matOpts.map = foliageTex;
                const mat = new THREE.MeshLambertMaterial(matOpts);
                group.add(block(blockGeo, mat, baseX + dx * 0.7 + jx, canopyBase + dy * 0.7, baseZ + dz * 0.7 + jz));
                count++;
            }
        }
    }

    // Branches latérales (quelques blocs qui dépassent)
    const branchGeo = new THREE.BoxGeometry(0.3, 0.3, 0.3);
    for (let b = 0; b < 2; b++) {
        const bx = (prng(id, b + 80) - 0.5) * 3.0;
        const bz = (prng(id, b + 82) - 0.5) * 3.0;
        const bMat = new THREE.MeshLambertMaterial({ color: trunkColor(isDying) });
        group.add(block(branchGeo, bMat, baseX + bx, canopyBase - 0.1, baseZ + bz));
    }

    // Fleurs si mature
    if (isMature) {
        const flowerGeo = new THREE.BoxGeometry(0.25, 0.25, 0.25);
        for (let f = 0; f < 3; f++) {
            const fx = (prng(id, f + 90) - 0.5) * 2.0;
            const fz = (prng(id, f + 92) - 0.5) * 2.0;
            const hue = prng(id, f + 95);
            const flowerMat = new THREE.MeshLambertMaterial({
                color: new THREE.Color().setHSL(hue, 0.9, 0.7),
                emissive: new THREE.Color().setHSL(hue, 0.5, 0.2),
            });
            group.add(block(flowerGeo, flowerMat, baseX + fx, canopyBase + 0.7 + 0.3, baseZ + fz));
        }
    }
}

/**
 * Arbre : tronc haut (3-6 blocs), canopée sphérique au sommet.
 * exudate=Nitrogen → canopée plate (parapluie).
 * exudate=Carbon → canopée ronde.
 * hidden_size > 10 → canopée plus détaillée.
 */
export function buildTree(group, plant, baseX, baseY, baseZ, color, params, hue) {
    const { biomass, isDying, isStressed, isMature, id, exudateType, hiddenSize } = params;
    const canopyColor = isDying
        ? color.clone().multiplyScalar(0.35)
        : isStressed
            ? color.clone().lerp(new THREE.Color(0x808060), 0.4)
            : color;

    // Tronc avec texture d'ecorce
    const trunkHeight = Math.min(5 + Math.floor(biomass / 6), 12);
    const trunkGeo = new THREE.BoxGeometry(0.6, 0.8, 0.6);
    const barkTex = barkTexture();
    const trunkMat = new THREE.MeshLambertMaterial({ map: barkTex, color: trunkColor(isDying) });

    for (let h = 0; h < trunkHeight; h++) {
        group.add(block(trunkGeo, trunkMat, baseX, baseY + h * 0.8 + 0.4, baseZ));
    }

    // Texture de feuillage parametree par la lignee
    const foliageTex = hue !== undefined ? foliageTexture(hue) : null;

    // Canopée (spheres organiques)
    const canopyBase = baseY + trunkHeight * 0.8;
    const isParapluie = exudateType === 'nitrogen';
    const detailed = hiddenSize > 10;
    const radiusBase = Math.max(2.0, Math.min(Math.sqrt(biomass) * 1.0, 4.5));
    const radius = isStressed ? radiusBase * 0.8 : isDying ? radiusBase * 0.5 : radiusBase;
    const canopyHeight = isParapluie ? 2 : Math.max(3, Math.floor(radius));
    const blockSize = detailed ? 1.0 : 1.2;
    const blockGeo = new THREE.SphereGeometry(blockSize * 0.5, 6, 6);
    let count = 0;

    const maxR = Math.ceil(radius);
    for (let dx = -maxR; dx <= maxR; dx++) {
        for (let dz = -maxR; dz <= maxR; dz++) {
            for (let dy = 0; dy < canopyHeight; dy++) {
                // Forme sphérique ou plate
                const hFactor = isParapluie ? 0.3 : 1.0;
                const dist = dx * dx + dz * dz + (dy * dy) / (hFactor * hFactor);
                if (dist > radius * radius) continue;
                if (count >= 25) break;

                const jx = (prng(id, count * 3 + 100) - 0.5) * 0.25;
                const jz = (prng(id, count * 3 + 101) - 0.5) * 0.25;
                const jy = detailed ? (prng(id, count * 3 + 102) - 0.5) * 0.2 : 0;
                const variation = prng(id, count + 200) * 0.12;
                const matOpts = { color: canopyColor.clone().offsetHSL(0, 0, variation - 0.06) };
                if (foliageTex) matOpts.map = foliageTex;
                const mat = new THREE.MeshLambertMaterial(matOpts);
                group.add(block(blockGeo, mat,
                    baseX + dx * blockSize + jx,
                    canopyBase + dy * blockSize + jy,
                    baseZ + dz * blockSize + jz
                ));
                count++;
            }
        }
    }

    // Fleurs si mature
    if (isMature) {
        const flowerGeo = new THREE.BoxGeometry(0.3, 0.3, 0.3);
        for (let f = 0; f < 4; f++) {
            const fx = (prng(id, f * 2 + 150) - 0.5) * radius * 1.5;
            const fz = (prng(id, f * 2 + 151) - 0.5) * radius * 1.5;
            const hue = prng(id, f + 160);
            const flowerMat = new THREE.MeshLambertMaterial({
                color: new THREE.Color().setHSL(hue, 0.9, 0.7),
                emissive: new THREE.Color().setHSL(hue, 0.5, 0.2),
            });
            group.add(block(flowerGeo, flowerMat,
                baseX + fx,
                canopyBase + canopyHeight * blockSize + 0.3,
                baseZ + fz
            ));
        }
    }
}

/**
 * Conifère : tronc fin et très haut, forme conique (pyramide).
 * Chaque étage de canopée est plus petit que le précédent.
 */
export function buildConifer(group, plant, baseX, baseY, baseZ, color, params, hue) {
    const { biomass, isDying, isStressed, isMature, id } = params;
    const canopyColor = isDying
        ? color.clone().multiplyScalar(0.3)
        : isStressed
            ? color.clone().lerp(new THREE.Color(0x506040), 0.4)
            : color.clone().offsetHSL(-0.05, -0.1, -0.05); // Vert plus sombre/bleuté

    // Texture de feuillage parametree par la lignee
    const foliageTex = hue !== undefined ? foliageTexture(hue) : null;

    // Tronc fin et haut avec texture d'ecorce
    const trunkHeight = Math.min(8 + Math.floor(biomass / 8), 15);
    const trunkGeo = new THREE.BoxGeometry(0.4, 0.8, 0.4);
    const barkTex = barkTexture();
    const trunkMat = new THREE.MeshLambertMaterial({ map: barkTex, color: trunkColor(isDying) });

    for (let h = 0; h < trunkHeight; h++) {
        group.add(block(trunkGeo, trunkMat, baseX, baseY + h * 0.8 + 0.4, baseZ));
    }

    // Canopée conique : etages decroissants avec cones hexagonaux
    const levels = Math.min(5 + Math.floor(biomass / 12), 8);
    const maxRadius = Math.max(2.0, Math.min(Math.sqrt(biomass) * 0.9, 3.5));
    const reducedRadius = isStressed ? maxRadius * 0.7 : isDying ? maxRadius * 0.4 : maxRadius;
    const blockGeo = new THREE.ConeGeometry(0.5, 0.8, 6);
    let count = 0;

    for (let level = 0; level < levels; level++) {
        // Rayon décroissant avec la hauteur (forme conique)
        const t = level / Math.max(levels - 1, 1);
        const levelRadius = reducedRadius * (1 - t * 0.8);
        const levelY = baseY + trunkHeight * 0.4 + level * 1.1;
        const r = Math.ceil(levelRadius);

        for (let dx = -r; dx <= r; dx++) {
            for (let dz = -r; dz <= r; dz++) {
                if (dx * dx + dz * dz > levelRadius * levelRadius) continue;
                if (count >= 28) break;

                const jx = (prng(id, count * 3 + 200) - 0.5) * 0.15;
                const jz = (prng(id, count * 3 + 201) - 0.5) * 0.15;
                const variation = prng(id, count + 300) * 0.1;
                const matOpts = { color: canopyColor.clone().offsetHSL(0, 0, variation - 0.05) };
                if (foliageTex) matOpts.map = foliageTex;
                const mat = new THREE.MeshLambertMaterial(matOpts);
                group.add(block(blockGeo, mat,
                    baseX + dx * 0.8 + jx,
                    levelY,
                    baseZ + dz * 0.8 + jz
                ));
                count++;
            }
        }
    }

    // Fleurs/pommes si mature
    if (isMature) {
        const coneGeo = new THREE.BoxGeometry(0.2, 0.3, 0.2);
        for (let f = 0; f < 3; f++) {
            const levelIdx = Math.floor(prng(id, f + 250) * levels);
            const t = levelIdx / Math.max(levels - 1, 1);
            const lr = reducedRadius * (1 - t * 0.8);
            const fx = (prng(id, f + 260) - 0.5) * lr * 1.5;
            const fz = (prng(id, f + 262) - 0.5) * lr * 1.5;
            const coneMat = new THREE.MeshLambertMaterial({
                color: new THREE.Color(0x8b4513),
                emissive: new THREE.Color(0x3a1a05),
            });
            group.add(block(coneGeo, coneMat,
                baseX + fx,
                baseY + trunkHeight * 0.4 + levelIdx * 1.1 + 0.4,
                baseZ + fz
            ));
        }
    }
}
