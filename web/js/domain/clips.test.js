import { describe, it, expect } from 'vitest';
import { ClipManager } from './clips.js';

describe('ClipManager', () => {
    it('demarre vide', () => {
        const cm = new ClipManager();
        expect(cm.clipCount()).toBe(0);
        expect(cm.currentClip()).toBeNull();
    });

    it('charge un montage', () => {
        const cm = new ClipManager();
        cm.loadMontage({
            clips: [
                { clip: { trigger: 'first_symbiosis' }, header: {}, events: [] },
                { clip: { trigger: 'fitness_record' }, header: {}, events: [] },
            ]
        });
        expect(cm.clipCount()).toBe(2);
        expect(cm.currentClip().clip.trigger).toBe('first_symbiosis');
    });

    it('navigue entre les clips', () => {
        const cm = new ClipManager();
        cm.loadMontage({
            clips: [
                { clip: { trigger: 'a' }, header: {}, events: [] },
                { clip: { trigger: 'b' }, header: {}, events: [] },
                { clip: { trigger: 'c' }, header: {}, events: [] },
            ]
        });
        expect(cm.currentClip().clip.trigger).toBe('a');
        cm.nextClip();
        expect(cm.currentClip().clip.trigger).toBe('b');
        cm.nextClip();
        expect(cm.currentClip().clip.trigger).toBe('c');
        cm.nextClip();
        expect(cm.currentClip().clip.trigger).toBe('c');  // reste au dernier
        cm.prevClip();
        expect(cm.currentClip().clip.trigger).toBe('b');
    });

    it('affiche les infos du clip courant', () => {
        const cm = new ClipManager();
        cm.loadMontage({ clips: [{ clip: {}, header: {}, events: [] }, { clip: {}, header: {}, events: [] }] });
        expect(cm.clipInfo()).toBe('Clip 1/2');
        cm.nextClip();
        expect(cm.clipInfo()).toBe('Clip 2/2');
    });
});
