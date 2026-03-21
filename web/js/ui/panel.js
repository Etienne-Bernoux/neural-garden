/**
 * Gestionnaire du panneau lateral.
 */
export class PanelManager {
    constructor() {
        this.tickEl = document.getElementById('tick');
        this.seasonEl = document.getElementById('season');
        this.populationEl = document.getElementById('population');
        this.lineagesEl = document.getElementById('lineages');
        this.symbiosisEl = document.getElementById('symbiosis');
        this.fitnessEl = document.getElementById('fitness');
        this.clipInfoEl = document.getElementById('clip-info');
        this.plantInfoEl = document.getElementById('plant-info');
        this.scrubEl = document.getElementById('scrub');
    }

    /**
     * Met a jour les stats globales.
     */
    updateStats(state) {
        if (this.tickEl) this.tickEl.textContent = state.tick;
        if (this.seasonEl) this.seasonEl.textContent = state.season;
        if (this.populationEl) this.populationEl.textContent = state.population;
        if (this.lineagesEl) this.lineagesEl.textContent = state.lineages;
        if (this.symbiosisEl) this.symbiosisEl.textContent = state.symbiosis;
        if (this.fitnessEl) this.fitnessEl.textContent = state.bestFitness.toFixed(1);
    }

    /**
     * Met a jour les infos de la plante selectionnee.
     */
    selectPlant(plant) {
        if (!plant) {
            if (this.plantInfoEl) this.plantInfoEl.style.display = 'none';
            return;
        }

        if (this.plantInfoEl) this.plantInfoEl.style.display = 'block';

        const setEl = (id, value) => {
            const el = document.getElementById(id);
            if (el) el.textContent = value;
        };

        setEl('plant-id', plant.id);
        setEl('plant-state', plant.state);
        setEl('plant-vitality', typeof plant.vitality === 'number' ? plant.vitality.toFixed(1) : plant.vitality);
        setEl('plant-energy', typeof plant.energy === 'number' ? plant.energy.toFixed(1) : plant.energy);
        setEl('plant-biomass', plant.biomass || plant.cells?.length || 0);
        setEl('plant-lineage', plant.lineage_id);
    }

    updateClipInfo(info) {
        if (this.clipInfoEl) this.clipInfoEl.textContent = info;
    }

    updateScrub(progress) {
        if (this.scrubEl) this.scrubEl.value = Math.floor(progress * 100);
    }
}
