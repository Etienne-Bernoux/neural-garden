import * as THREE from 'three';

// Couleur du sol par altitude
function terrainColor(altitude, humidity) {
    const h = 0.08 + altitude * 0.08;  // teinte brun -> vert
    const s = 0.4 + humidity * 0.3;
    const l = 0.2 + altitude * 0.3;
    return new THREE.Color().setHSL(h, s, l);
}

/**
 * Crée le mesh du terrain à partir des données d'altitude.
 * @param {object} header - Données du header (grid_size, altitude[][])
 * @returns {THREE.Group} - Groupe contenant le terrain + l'eau
 */
export function createTerrain(header) {
    const group = new THREE.Group();
    const gridSize = header.grid_size || 128;

    // Géométrie du terrain : une box par cellule
    // Pour la performance, on utilise InstancedMesh
    const blockGeo = new THREE.BoxGeometry(1, 1, 1);
    const blockMat = new THREE.MeshLambertMaterial({ vertexColors: true });

    // Compter les blocs nécessaires
    let blockCount = 0;
    const altitudes = header.altitude || [];
    for (let y = 0; y < gridSize; y++) {
        for (let x = 0; x < gridSize; x++) {
            const alt = altitudes[y] ? (altitudes[y][x] || 0) : 0;
            const height = Math.max(1, Math.floor(alt * 10));
            blockCount += height;
        }
    }

    // Créer l'InstancedMesh
    const mesh = new THREE.InstancedMesh(blockGeo, blockMat, blockCount);
    const matrix = new THREE.Matrix4();
    const color = new THREE.Color();
    let idx = 0;

    for (let y = 0; y < gridSize; y++) {
        for (let x = 0; x < gridSize; x++) {
            const alt = altitudes[y] ? (altitudes[y][x] || 0) : 0;
            const humidity = header.initial_humidity ? (header.initial_humidity[y]?.[x] || 0) : 0.5;
            const height = Math.max(1, Math.floor(alt * 10));

            for (let h = 0; h < height; h++) {
                matrix.setPosition(x - gridSize / 2, h, y - gridSize / 2);
                mesh.setMatrixAt(idx, matrix);

                // Couleur : le bloc du dessus est plus clair
                const isTop = (h === height - 1);
                const baseColor = terrainColor(alt, humidity);
                if (isTop) {
                    baseColor.multiplyScalar(1.2);
                } else {
                    baseColor.multiplyScalar(0.7);
                }
                mesh.setColorAt(idx, baseColor);
                idx++;
            }
        }
    }

    mesh.instanceMatrix.needsUpdate = true;
    if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
    group.add(mesh);

    // Plan d'eau (mer)
    const waterGeo = new THREE.PlaneGeometry(gridSize * 2, gridSize * 2);
    const waterMat = new THREE.MeshLambertMaterial({
        color: 0x1a5276,
        transparent: true,
        opacity: 0.7,
    });
    const water = new THREE.Mesh(waterGeo, waterMat);
    water.rotation.x = -Math.PI / 2;
    water.position.set(0, 0.5, 0);  // juste au-dessus du niveau 0
    group.add(water);

    return group;
}
