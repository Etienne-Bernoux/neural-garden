/**
 * Gestionnaire d'etat de la simulation.
 * Reconstruit l'etat a partir d'un header + events (replay)
 * ou depuis un snapshot (mode live).
 */
export class SimState {
    constructor() {
        this.plants = new Map();  // id -> plant data
        this.links = [];          // liens mycorhiziens actifs
        this.season = 'Spring';
        this.tick = 0;
        this.population = 0;
        this.lineages = 0;
        this.symbiosis = 0;
        this.bestFitness = 0;
        this.terrainHeights = null;
        this.gridSize = 128;
    }

    /**
     * Charge l'etat initial depuis un header de clip.
     */
    loadHeader(header) {
        this.gridSize = header.grid_size || 128;
        this.terrainHeights = header.altitude || [];
        this.plants.clear();
        this.links = [];
        this.tick = 0;
        this.bestFitness = 0;
        this.season = 'Spring';
        this.lastInvasion = null;

        if (header.plants) {
            for (const p of header.plants) {
                this.plants.set(p.id, {
                    id: p.id,
                    lineage_id: p.lineage_id,
                    cells: p.cells || [],
                    vitality: p.vitality || 100,
                    energy: p.energy || 50,
                    biomass: p.cells ? p.cells.length : 1,
                    state: 'Growing',
                    traits: p.traits || {},
                });
            }
        }

        this._updateCounts();
    }

    /**
     * Charge un snapshot complet (mode live).
     */
    loadSnapshot(snapshot) {
        this.tick = snapshot.tick || 0;
        this.season = snapshot.season || 'Spring';
        this.gridSize = snapshot.grid_size || 128;
        this.terrainHeights = snapshot.terrain_heights || [];
        this.bestFitness = snapshot.best_fitness || 0;

        this.plants.clear();
        if (snapshot.plants) {
            for (const p of snapshot.plants) {
                this.plants.set(p.id, { ...p });
            }
        }

        this.links = snapshot.links || [];
        this._updateCounts();
    }

    /**
     * Applique un event incremental.
     */
    applyEvent(event) {
        switch (event.event_type || event.e) {
            case 'grow':
                this._handleGrow(event);
                break;
            case 'shrink':
                this._handleShrink(event);
                break;
            case 'born':
                this._handleBorn(event);
                break;
            case 'died':
                this._handleDied(event);
                break;
            case 'germinate':
                this._handleGerminate(event);
                break;
            case 'link':
                this._handleLink(event);
                break;
            case 'unlink':
                this._handleUnlink(event);
                break;
            case 'invade':
                this._handleInvade(event);
                break;
            case 'season':
                this.season = event.data?.name || event.name || this.season;
                break;
        }

        if (event.t || event.tick) {
            this.tick = event.t || event.tick;
        }

        this._updateCounts();
    }

    _handleGrow(e) {
        const data = e.data || e;
        const id = data.plant_id || data.p;
        const plant = this.plants.get(id);
        if (plant) {
            const x = data.x ?? data.cell?.[0];
            const y = data.y ?? data.cell?.[1];
            if (x !== undefined && y !== undefined) {
                plant.cells.push([x, y]);
                plant.biomass = plant.cells.length;
            }
        }
    }

    _handleShrink(e) {
        const data = e.data || e;
        const id = data.plant_id || data.p;
        const plant = this.plants.get(id);
        if (plant && plant.cells.length > 0) {
            const x = data.x ?? data.cell?.[0];
            const y = data.y ?? data.cell?.[1];
            if (x !== undefined && y !== undefined) {
                const idx = plant.cells.findIndex(c => c[0] === x && c[1] === y);
                if (idx >= 0) plant.cells.splice(idx, 1);
                plant.biomass = plant.cells.length;
            }
        }
    }

    _handleBorn(e) {
        const data = e.data || e;
        const id = data.plant_id || data.p;
        const x = data.x ?? data.position?.[0];
        const y = data.y ?? data.position?.[1];
        this.plants.set(id, {
            id,
            lineage_id: data.lineage_id || data.lin || 0,
            cells: (x !== undefined && y !== undefined) ? [[x, y]] : [],
            vitality: 100,
            energy: 50,
            biomass: 1,
            state: 'Seed',
            traits: {},
        });
    }

    _handleDied(e) {
        const data = e.data || e;
        const id = data.plant_id || data.p;
        const plant = this.plants.get(id);
        if (plant) {
            plant.state = 'Dead';
        }
    }

    _handleGerminate(e) {
        const data = e.data || e;
        const id = data.plant_id || data.p;
        const plant = this.plants.get(id);
        if (plant) {
            plant.state = 'Growing';
        }
    }

    _handleLink(e) {
        const data = e.data || e;
        const a = data.plant_a || data.a;
        const b = data.plant_b || data.b;
        const plantA = this.plants.get(a);
        const plantB = this.plants.get(b);
        this.links.push({
            plant_a: a,
            plant_b: b,
            pos_a: plantA?.cells[0],
            pos_b: plantB?.cells[0],
        });
    }

    _handleUnlink(e) {
        const data = e.data || e;
        const a = data.plant_a || data.a;
        const b = data.plant_b || data.b;
        this.links = this.links.filter(l =>
            !(l.plant_a === a && l.plant_b === b) &&
            !(l.plant_a === b && l.plant_b === a)
        );
    }

    _handleInvade(e) {
        const data = e.data || e;
        const invaderId = data.invader_id || data.p;
        const victimId = data.victim_id || data.victim;
        const x = data.x ?? data.cell?.[0];
        const y = data.y ?? data.cell?.[1];

        // Retirer la cellule de la victime
        const victim = this.plants.get(victimId);
        if (victim && x !== undefined && y !== undefined) {
            const idx = victim.cells.findIndex(c => c[0] === x && c[1] === y);
            if (idx >= 0) victim.cells.splice(idx, 1);
            victim.biomass = victim.cells.length;
        }

        // Ajouter a l'envahisseur
        const invader = this.plants.get(invaderId);
        if (invader && x !== undefined && y !== undefined) {
            invader.cells.push([x, y]);
            invader.biomass = invader.cells.length;
        }

        // Marquer pour flash
        this.lastInvasion = { x, y };
    }

    _updateCounts() {
        let alive = 0;
        const lineageSet = new Set();
        for (const [, p] of this.plants) {
            if (p.state !== 'Dead' && p.state !== 'Decomposing') {
                alive++;
                lineageSet.add(p.lineage_id);
            }
        }
        this.population = alive;
        this.lineages = lineageSet.size;
        this.symbiosis = this.links.length;
    }

    /**
     * Retourne la liste des plantes vivantes pour le rendu.
     */
    getAlivePlants() {
        const result = [];
        for (const [, p] of this.plants) {
            if (p.state !== 'Dead' && p.state !== 'Decomposing') {
                result.push(p);
            }
        }
        return result;
    }
}
