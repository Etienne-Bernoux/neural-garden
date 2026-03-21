/**
 * Gestionnaire de timeline pour le replay.
 */
export class Timeline {
    constructor() {
        this.playing = false;
        this.speed = 1;
        this.currentEventIndex = 0;
        this.events = [];
        this.tickStart = 0;
        this.tickEnd = 0;
    }

    loadClip(clip) {
        this.events = clip.events || [];
        this.tickStart = clip.clip?.tick_start || (this.events.length > 0 ? (this.events[0].t || 0) : 0);
        this.tickEnd = clip.clip?.tick_end || (this.events.length > 0 ? (this.events[this.events.length - 1].t || 0) : 0);
        this.currentEventIndex = 0;
    }

    togglePlay() { this.playing = !this.playing; }
    pause() { this.playing = false; }
    play() { this.playing = true; }

    setSpeed(speed) { this.speed = Math.max(0.25, Math.min(4, speed)); }

    /**
     * Avance la timeline et retourne les events a appliquer.
     */
    advance() {
        if (!this.playing) return [];

        const eventsToApply = [];
        const eventsPerFrame = Math.max(1, Math.floor(this.speed));

        for (let i = 0; i < eventsPerFrame && this.currentEventIndex < this.events.length; i++) {
            eventsToApply.push(this.events[this.currentEventIndex]);
            this.currentEventIndex++;
        }

        // Si on a atteint la fin, pause
        if (this.currentEventIndex >= this.events.length) {
            this.playing = false;
        }

        return eventsToApply;
    }

    /**
     * Scrub a une position (0-1).
     */
    scrubTo(ratio) {
        this.currentEventIndex = Math.floor(ratio * this.events.length);
    }

    progress() {
        return this.events.length > 0 ? this.currentEventIndex / this.events.length : 0;
    }

    isFinished() {
        return this.currentEventIndex >= this.events.length;
    }
}
