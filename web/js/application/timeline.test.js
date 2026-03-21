import { describe, it, expect } from 'vitest';
import { Timeline } from './timeline.js';

describe('Timeline', () => {
    it('demarre en pause', () => {
        const tl = new Timeline();
        expect(tl.playing).toBe(false);
        expect(tl.progress()).toBe(0);
    });

    it('charge un clip', () => {
        const tl = new Timeline();
        tl.loadClip({
            clip: { tick_start: 100, tick_end: 200 },
            events: [{ t: 101 }, { t: 102 }, { t: 103 }]
        });
        expect(tl.events).toHaveLength(3);
        expect(tl.tickStart).toBe(100);
    });

    it('toggle play/pause', () => {
        const tl = new Timeline();
        tl.togglePlay();
        expect(tl.playing).toBe(true);
        tl.togglePlay();
        expect(tl.playing).toBe(false);
    });

    it('avance et retourne des events', () => {
        const tl = new Timeline();
        tl.loadClip({ events: [{ t: 1, e: 'a' }, { t: 2, e: 'b' }] });
        tl.play();
        const events = tl.advance();
        expect(events).toHaveLength(1);
        expect(events[0].e).toBe('a');
    });

    it('ne retourne rien en pause', () => {
        const tl = new Timeline();
        tl.loadClip({ events: [{ t: 1, e: 'a' }] });
        expect(tl.advance()).toHaveLength(0);
    });

    it('auto-pause en fin de clip', () => {
        const tl = new Timeline();
        tl.loadClip({ events: [{ t: 1, e: 'a' }] });
        tl.play();
        tl.advance();
        expect(tl.playing).toBe(false);
        expect(tl.isFinished()).toBe(true);
    });

    it('scrub a une position', () => {
        const tl = new Timeline();
        tl.loadClip({ events: new Array(100).fill(null).map((_, i) => ({ t: i })) });
        tl.scrubTo(0.5);
        expect(tl.currentEventIndex).toBe(50);
    });

    it('respecte la vitesse', () => {
        const tl = new Timeline();
        tl.loadClip({ events: new Array(10).fill(null).map((_, i) => ({ t: i })) });
        tl.setSpeed(3);
        tl.play();
        expect(tl.advance()).toHaveLength(3);
    });
});
