import { describe, it, expect } from 'vitest';
import { SimState } from './state.js';

describe('SimState', () => {
    it('demarre avec un etat vide', () => {
        const state = new SimState();
        expect(state.population).toBe(0);
        expect(state.plants.size).toBe(0);
        expect(state.links).toHaveLength(0);
    });

    it('charge un header avec des plantes', () => {
        const state = new SimState();
        state.loadHeader({
            grid_size: 128,
            altitude: [],
            plants: [
                { id: 1, lineage_id: 0, cells: [[10, 10]], vitality: 80, energy: 50 },
                { id: 2, lineage_id: 1, cells: [[20, 20]], vitality: 60, energy: 30 },
            ]
        });
        expect(state.plants.size).toBe(2);
        expect(state.population).toBe(2);
        expect(state.lineages).toBe(2);
    });

    it('ajoute une plante via event born', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        expect(state.plants.size).toBe(1);
        // Les graines ne comptent pas dans population (seulement apres germination)
        expect(state.population).toBe(0);
        expect(state.plants.get(1).state).toBe('Seed');
    });

    it('change etat via event germinate', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'germinate', p: 1 });
        expect(state.plants.get(1).state).toBe('Growing');
    });

    it('ajoute une cellule via event grow (footprint par defaut)', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'grow', p: 1, x: 11, y: 10 });
        expect(state.plants.get(1).footprint).toHaveLength(2);
        expect(state.plants.get(1).biomass).toBe(2);
    });

    it('retire une cellule via event shrink', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'grow', p: 1, x: 11, y: 10 });
        state.applyEvent({ e: 'shrink', p: 1, x: 11, y: 10 });
        expect(state.plants.get(1).footprint).toHaveLength(1);
    });

    it('marque une plante morte via event died', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'died', p: 1 });
        expect(state.plants.get(1).state).toBe('Dead');
        expect(state.population).toBe(0);
    });

    it('cree un lien via event link', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'born', p: 2, lin: 1, x: 12, y: 10 });
        state.applyEvent({ e: 'link', a: 1, b: 2 });
        expect(state.links).toHaveLength(1);
        expect(state.symbiosis).toBe(1);
    });

    it('supprime un lien via event unlink', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'born', p: 2, lin: 1, x: 12, y: 10 });
        state.applyEvent({ e: 'link', a: 1, b: 2 });
        state.applyEvent({ e: 'unlink', a: 1, b: 2 });
        expect(state.links).toHaveLength(0);
    });

    it('transfere une cellule via event invade', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'born', p: 2, lin: 1, x: 11, y: 10 });
        state.applyEvent({ e: 'invade', p: 1, victim: 2, x: 11, y: 10 });
        expect(state.plants.get(1).footprint).toHaveLength(2);
        expect(state.plants.get(2).footprint).toHaveLength(0);
    });

    it('change la saison via event season', () => {
        const state = new SimState();
        state.applyEvent({ e: 'season', name: 'Winter' });
        expect(state.season).toBe('Winter');
    });

    it('exclut les morts et les graines de getAlivePlants', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.applyEvent({ e: 'born', p: 2, lin: 1, x: 20, y: 20 });
        state.applyEvent({ e: 'germinate', p: 2 });  // plante 2 germe → visible
        state.applyEvent({ e: 'died', p: 1 });
        const alive = state.getAlivePlants();
        expect(alive).toHaveLength(1);  // plante 1 morte, plante 2 visible (Growing)
        expect(alive[0].id).toBe(2);
    });

    it('loadHeader reinitialise les champs transients', () => {
        const state = new SimState();
        state.tick = 500;
        state.bestFitness = 123.4;
        state.season = 'Winter';
        state.loadHeader({ grid_size: 128, altitude: [], plants: [] });
        expect(state.tick).toBe(0);
        expect(state.bestFitness).toBe(0);
        expect(state.season).toBe('Spring');
    });

    it('remplace letat complet via loadSnapshot', () => {
        const state = new SimState();
        state.applyEvent({ e: 'born', p: 1, lin: 0, x: 10, y: 10 });
        state.loadSnapshot({
            tick: 500,
            season: 'Autumn',
            grid_size: 128,
            terrain_heights: [],
            plants: [{ id: 5, lineage_id: 3, cells: [[30, 30]], vitality: 90, energy: 40, biomass: 1, state: 'Growing' }],
            links: [],
            best_fitness: 123.4,
        });
        expect(state.tick).toBe(500);
        expect(state.season).toBe('Autumn');
        expect(state.plants.size).toBe(1);
        expect(state.plants.has(1)).toBe(false);
        expect(state.plants.has(5)).toBe(true);
    });

    it('dispatche la croissance par couche', () => {
        const state = new SimState();
        state.applyEvent({ event_type: 'Born', data: { plant_id: 1, lineage_id: 0, x: 10, y: 10 } });
        state.applyEvent({ event_type: 'Grew', data: { plant_id: 1, x: 11, y: 10, layer: 'Footprint' } });
        state.applyEvent({ event_type: 'Grew', data: { plant_id: 1, x: 12, y: 10, layer: 'Canopy' } });
        state.applyEvent({ event_type: 'Grew', data: { plant_id: 1, x: 13, y: 10, layer: 'Roots' } });
        expect(state.plants.get(1).footprint).toHaveLength(2);  // 1 initiale + 1 grew
        expect(state.plants.get(1).cells).toHaveLength(1);       // 1 canopy grew (pas l'initiale)
        expect(state.plants.get(1).roots).toHaveLength(2);       // 1 initiale + 1 grew
    });

    // Tests format PascalCase (DomainEventDto du serveur live)
    it('accepte les events en PascalCase (format live)', () => {
        const state = new SimState();
        state.applyEvent({ event_type: 'Born', data: { plant_id: 1, lineage_id: 0, x: 10, y: 10 } });
        expect(state.plants.size).toBe(1);

        state.applyEvent({ event_type: 'Germinated', data: { plant_id: 1 } });
        expect(state.plants.get(1).state).toBe('Growing');

        state.applyEvent({ event_type: 'Grew', data: { plant_id: 1, x: 11, y: 10, layer: 'Footprint' } });
        expect(state.plants.get(1).footprint).toHaveLength(2);

        state.applyEvent({ event_type: 'Died', data: { plant_id: 1 } });
        expect(state.plants.get(1).state).toBe('Dead');
    });
});
