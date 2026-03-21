/**
 * Gestionnaire de clips du montage.
 */
export class ClipManager {
    constructor() {
        this.clips = [];
        this.currentClipIndex = 0;
        this.metadata = null;
    }

    /**
     * Charge un montage JSON.
     */
    loadMontage(montage) {
        this.metadata = montage.metadata || {};
        this.clips = montage.clips || [];
        this.currentClipIndex = 0;
    }

    currentClip() {
        return this.clips[this.currentClipIndex] || null;
    }

    nextClip() {
        if (this.currentClipIndex < this.clips.length - 1) {
            this.currentClipIndex++;
        }
        return this.currentClip();
    }

    prevClip() {
        if (this.currentClipIndex > 0) {
            this.currentClipIndex--;
        }
        return this.currentClip();
    }

    clipCount() {
        return this.clips.length;
    }

    clipInfo() {
        return `Clip ${this.currentClipIndex + 1}/${this.clips.length}`;
    }
}
