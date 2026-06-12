import { Player } from '@rlview/framework';

export class ReplayLoader {
    /**
     * Load a replay file (JSON or binary) from URL
     * Uses the unified Player.load() API
     * Data is returned in raw Unreal Units
     * @param {string} url - URL to the replay file (.json or .bin)
     */
    async load(url) {
        try {
            const isBinary = url.endsWith('.bin');
            console.log(`[ReplayLoader] Loading ${isBinary ? 'binary' : 'JSON'} replay...`);

            const response = await fetch(url);

            if (isBinary) {
                const arrayBuffer = await response.arrayBuffer();
                console.log(`[ReplayLoader] Binary loaded (${(arrayBuffer.byteLength / 1024 / 1024).toFixed(2)} MB)`);
                const player = await Player.load(arrayBuffer);
                console.log("[ReplayLoader] Player initialized from binary");
                return player;
            } else {
                const replayData = await response.json();
                console.log("[ReplayLoader] JSON loaded, compiling...");
                const player = await Player.load(replayData);
                console.log("[ReplayLoader] Player initialized from JSON");
                return player;
            }
        } catch (error) {
            console.error("Error loading replay:", error);
            return null;
        }
    }

    /**
     * Load a replay from binary data (ArrayBuffer)
     * Used when binary data is already loaded (e.g., from API)
     * Data is returned in raw Unreal Units
     * @param {ArrayBuffer} arrayBuffer - Binary replay data
     */
    async loadFromBinary(arrayBuffer) {
        try {
            console.log(`[ReplayLoader] Loading from binary ArrayBuffer (${(arrayBuffer.byteLength / 1024 / 1024).toFixed(2)} MB)`);
            const player = await Player.load(arrayBuffer);
            console.log("[ReplayLoader] Player initialized from binary ArrayBuffer");
            return player;
        } catch (error) {
            console.error("Error loading replay from binary:", error);
            return null;
        }
    }
}
