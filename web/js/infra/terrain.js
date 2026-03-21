import * as THREE from 'three';

// Echelle verticale du terrain — partagee avec plants.js et symbiosis.js
export const TERRAIN_SCALE = 15;

// Couleur du sol par altitude
function terrainColor(altitude, humidity) {
    if (altitude < 0.05) return new THREE.Color(0xc2b280);  // sable
    if (altitude < 0.15) return new THREE.Color(0x7caa2d);  // herbe basse
    if (altitude < 0.4) return new THREE.Color(0x4a7c1f);   // herbe
    if (altitude < 0.6) return new THREE.Color(0x3a6b1a);   // foret
    if (altitude < 0.8) return new THREE.Color(0x6b5b3a);   // roche
    return new THREE.Color(0x8b7d6b);                        // sommet
}

/**
 * Cree le mesh du terrain a partir des donnees d'altitude.
 * Utilise un PlaneGeometry avec displacement (une seule mesh).
 * @param {object} header - Donnees du header (grid_size, altitude[][])
 * @returns {THREE.Group} - Groupe contenant le terrain + l'eau
 */
export function createTerrain(header) {
    const group = new THREE.Group();
    const gridSize = header.grid_size || 128;
    const altitudes = header.altitude || [];

    // Terrain : PlaneGeometry avec vertices deplacees en Y
    const geo = new THREE.PlaneGeometry(gridSize, gridSize, gridSize - 1, gridSize - 1);
    geo.rotateX(-Math.PI / 2);

    const positions = geo.attributes.position;
    const colors = new Float32Array(positions.count * 3);

    const seaThreshold = 0.3;  // meme seuil que le niveau de la mer
    const waterColor = new THREE.Color(0x1a6b8a);

    for (let i = 0; i < positions.count; i++) {
        const x = Math.round(positions.getX(i) + gridSize / 2);
        const z = Math.round(positions.getZ(i) + gridSize / 2);

        const gx = Math.min(Math.max(x, 0), gridSize - 1);
        const gz = Math.min(Math.max(z, 0), gridSize - 1);

        const alt = altitudes[gz] ? (altitudes[gz][gx] || 0) : 0;

        // Les cellules sous la mer sont aplaties au niveau de la mer
        const isUnderwater = alt <= seaThreshold;
        const height = isUnderwater ? seaThreshold * TERRAIN_SCALE : alt * TERRAIN_SCALE;

        positions.setY(i, height);

        // Couleur : sous-marin = couleur de l'eau, terrestre = par altitude
        if (isUnderwater) {
            colors[i * 3] = waterColor.r;
            colors[i * 3 + 1] = waterColor.g;
            colors[i * 3 + 2] = waterColor.b;
        } else {
            const humidity = header.initial_humidity ? (header.initial_humidity[gz]?.[gx] || 0.5) : 0.5;
            const color = terrainColor(alt, humidity);
            colors[i * 3] = color.r;
            colors[i * 3 + 1] = color.g;
            colors[i * 3 + 2] = color.b;
        }
    }

    geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
    geo.computeVertexNormals();

    const mat = new THREE.MeshLambertMaterial({ vertexColors: true });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.receiveShadow = true;
    group.add(mesh);

    // Eau : plan au niveau de la mer
    const seaLevel = 0.3 * TERRAIN_SCALE;
    const waterGeo = new THREE.PlaneGeometry(gridSize * 20, gridSize * 20);
    const waterMat = new THREE.MeshPhongMaterial({
        color: 0x1a6b8a,
        transparent: true,
        opacity: 0.6,
        shininess: 100,
    });
    const water = new THREE.Mesh(waterGeo, waterMat);
    water.rotation.x = -Math.PI / 2;
    water.position.y = seaLevel;
    group.add(water);

    return group;
}
