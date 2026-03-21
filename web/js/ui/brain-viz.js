/**
 * Visualisation du cerveau d'une plante sur un canvas 2D.
 * Réseau 18→H→H→8 avec valeurs colorées.
 */
export class BrainViz {
    constructor() {
        this.canvas = null;
        this.ctx = null;
        this.visible = false;
        this._createCanvas();
    }

    _createCanvas() {
        // Créer le conteneur dans le panneau latéral
        const panel = document.getElementById('panel');
        if (!panel) return;

        const container = document.createElement('div');
        container.id = 'brain-viz-container';
        container.style.display = 'none';
        container.innerHTML = '<h3>Cerveau</h3>';

        this.canvas = document.createElement('canvas');
        this.canvas.id = 'brain-canvas';
        this.canvas.width = 250;
        this.canvas.height = 300;
        this.canvas.style.width = '100%';
        this.canvas.style.background = '#0a0a1a';
        this.canvas.style.borderRadius = '4px';

        container.appendChild(this.canvas);
        panel.appendChild(container);

        this.ctx = this.canvas.getContext('2d');
    }

    toggle() {
        this.visible = !this.visible;
        const container = document.getElementById('brain-viz-container');
        if (container) {
            container.style.display = this.visible ? 'block' : 'none';
        }
    }

    /**
     * Dessiner le réseau de la plante sélectionnée.
     * @param {object} plant - la plante avec ses traits
     * @param {number[]} inputs - les 18 inputs (optionnel, peut être null)
     * @param {number[]} outputs - les 8 outputs (optionnel)
     */
    draw(plant, inputs, outputs) {
        if (!this.ctx || !this.visible) return;

        const ctx = this.ctx;
        const w = this.canvas.width;
        const h = this.canvas.height;

        ctx.clearRect(0, 0, w, h);

        if (!plant) {
            ctx.fillStyle = '#666';
            ctx.font = '12px monospace';
            ctx.textAlign = 'center';
            ctx.fillText('Selectionnez une plante', w / 2, h / 2);
            return;
        }

        const hiddenSize = plant.traits?.hidden_size || 8;

        // Layout des couches
        const layers = [
            { size: 18, x: 30, label: 'Inputs' },
            { size: hiddenSize, x: 95, label: 'H1' },
            { size: hiddenSize, x: 155, label: 'H2' },
            { size: 8, x: 220, label: 'Outputs' },
        ];

        // Labels des inputs
        const inputLabels = [
            'vit', 'ene', 'bio', 'age',
            'C', 'N', 'H\u2082O', 'lum',
            '\u2207Cx', '\u2207Cy', '\u2207Nx', '\u2207Ny',
            '\u2207Hx', '\u2207Hy', '\u2207Bx', '\u2207By',
            '\u2207Lx', '\u2207Ly',
        ];

        // Labels des outputs
        const outputLabels = [
            'grow', 'dir_x', 'dir_y', 'can/rac',
            'exud', 'def', 'conn', 'gen',
        ];

        // Dessiner les connexions (lignes légères)
        ctx.strokeStyle = 'rgba(100, 100, 150, 0.15)';
        ctx.lineWidth = 0.5;
        for (let l = 0; l < layers.length - 1; l++) {
            const from = layers[l];
            const to = layers[l + 1];
            for (let i = 0; i < from.size; i++) {
                const y1 = this._neuronY(i, from.size, h);
                for (let j = 0; j < to.size; j++) {
                    const y2 = this._neuronY(j, to.size, h);
                    ctx.beginPath();
                    ctx.moveTo(from.x, y1);
                    ctx.lineTo(to.x, y2);
                    ctx.stroke();
                }
            }
        }

        // Dessiner les neurones
        for (let l = 0; l < layers.length; l++) {
            const layer = layers[l];
            const values = l === 0 ? inputs : (l === 3 ? outputs : null);

            for (let i = 0; i < layer.size; i++) {
                const y = this._neuronY(i, layer.size, h);
                const val = values ? values[i] : null;

                // Couleur du neurone selon la valeur
                let fillColor = '#334';
                if (val !== null && val !== undefined) {
                    if (val > 0) {
                        const intensity = Math.min(val, 1.0);
                        fillColor = `rgb(${Math.floor(50 + intensity * 150)}, ${Math.floor(150 + intensity * 105)}, ${Math.floor(50 + intensity * 50)})`;
                    } else {
                        const intensity = Math.min(Math.abs(val), 1.0);
                        fillColor = `rgb(${Math.floor(150 + intensity * 105)}, ${Math.floor(50 + intensity * 50)}, ${Math.floor(50 + intensity * 50)})`;
                    }
                }

                // Dessiner le cercle
                const radius = layer.size > 12 ? 3 : 5;
                ctx.beginPath();
                ctx.arc(layer.x, y, radius, 0, Math.PI * 2);
                ctx.fillStyle = fillColor;
                ctx.fill();
                ctx.strokeStyle = '#556';
                ctx.lineWidth = 0.5;
                ctx.stroke();
            }

            // Label de la couche
            ctx.fillStyle = '#888';
            ctx.font = '9px monospace';
            ctx.textAlign = 'center';
            ctx.fillText(layer.label, layer.x, h - 5);
        }

        // Labels des inputs (à gauche)
        ctx.fillStyle = '#777';
        ctx.font = '7px monospace';
        ctx.textAlign = 'right';
        for (let i = 0; i < Math.min(inputLabels.length, 18); i++) {
            const y = this._neuronY(i, 18, h);
            ctx.fillText(inputLabels[i], layers[0].x - 5, y + 2);
        }

        // Labels des outputs (à droite)
        ctx.textAlign = 'left';
        for (let i = 0; i < outputLabels.length; i++) {
            const y = this._neuronY(i, 8, h);
            ctx.fillText(outputLabels[i], layers[3].x + 8, y + 2);
        }

        // Info hidden_size
        ctx.fillStyle = '#555';
        ctx.font = '8px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(`hidden: ${hiddenSize}`, w / 2, 12);
    }

    /**
     * Position Y d'un neurone dans une couche.
     */
    _neuronY(index, layerSize, canvasHeight) {
        const margin = 20;
        const available = canvasHeight - margin * 2;
        if (layerSize <= 1) return canvasHeight / 2;
        return margin + (index / (layerSize - 1)) * available;
    }
}
