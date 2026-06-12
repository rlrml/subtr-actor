/**
 * ClipRecordingManager
 *
 * Manages real-time camera recording during clip creation.
 * Captures camera position, rotation, mode, and target at 60 FPS.
 *
 * Feature: 024-clip-system (US1 - Capture Mode)
 */

export class ClipRecordingManager {
  constructor() {
    // Recording state
    this.isRecording = false;
    this.frames = [];

    // Timing
    this.startTime = 0;       // Game time when recording started
    this.recordingDuration = 0; // Total recording duration in ms
    this.sampleRate = 60;     // Frames per second

    // Last captured time to avoid duplicate frames
    this.lastCaptureTime = -1;

    // Accumulated time for frame timing (ensures consistent 60 FPS sampling)
    this.accumulatedTime = 0;
    this.frameInterval = 1000 / this.sampleRate; // ~16.67ms

    // Player list for name to index conversion
    this.playerList = null;
  }

  /**
   * Set the player list for name to index conversion
   * @param {Array} playerList - Array of player info objects with name property
   */
  setPlayerList(playerList) {
    this.playerList = playerList;
  }

  /**
   * Start recording camera frames
   * @param {number} gameTime - Current game time in seconds
   */
  start(gameTime) {
    this.isRecording = true;
    this.frames = [];
    this.startTime = gameTime;
    this.lastCaptureTime = -1;
    this.accumulatedTime = 0;
    this.recordingDuration = 0;

    console.log('[ClipRecordingManager] Recording started at game time:', gameTime);
  }

  /**
   * Stop recording
   * @returns {Object} Recording data with type, sampleRate, duration, and frames
   */
  stop() {
    this.isRecording = false;

    const data = {
      type: 'capture',
      sampleRate: this.sampleRate,
      duration: Math.round(this.recordingDuration),
      frames: this.frames,
    };

    console.log('[ClipRecordingManager] Recording stopped. Frames:', this.frames.length, 'Duration:', this.recordingDuration, 'ms');

    return data;
  }

  /**
   * Capture a frame from the current camera state
   * Called from GameEngine.animate() loop
   *
   * NOTE (026-clip-editor-redesign): Frame timestamps are RELATIVE to recording start.
   * This matches how ClipPlaybackManager expects them.
   *
   * @param {number} delta - Delta time since last frame in seconds
   * @param {Object} cameraState - Current camera state from GameEngine.getCameraState()
   * @param {number} gameTime - Current game time in seconds
   */
  captureFrame(delta, cameraState, gameTime) {
    if (!this.isRecording) return;

    // Accumulate time
    this.accumulatedTime += delta * 1000; // Convert to ms

    // Check if we should capture a frame (based on sampleRate)
    if (this.accumulatedTime < this.frameInterval) {
      return;
    }

    // Reset accumulated time (keep remainder for precision)
    this.accumulatedTime = this.accumulatedTime % this.frameInterval;

    // Calculate time offset from recording start (RELATIVE time in ms)
    // Using precise calculation without additional rounding to prevent drift
    const timeOffset = Math.round((gameTime - this.startTime) * 1000);

    // Don't capture duplicate frames at the same time
    if (timeOffset <= this.lastCaptureTime) {
      return;
    }
    this.lastCaptureTime = timeOffset;

    // Map camera mode to compact format
    let m = 'f'; // free
    if (cameraState.mode === 'ball' || cameraState.mode === 'ballOrbit') {
      m = 'b'; // ball
    } else if (cameraState.mode === 'player') {
      m = 'p'; // player
    }

    // Create frame data with compact field names
    const frame = {
      t: timeOffset,
      px: this._round(cameraState.position.x, 2),
      py: this._round(cameraState.position.y, 2),
      pz: this._round(cameraState.position.z, 2),
      qx: this._round(cameraState.rotation.x, 4),
      qy: this._round(cameraState.rotation.y, 4),
      qz: this._round(cameraState.rotation.z, 4),
      qw: this._round(cameraState.rotation.w, 4),
      m,
    };

    // Add target player index if in player cam mode
    if (m === 'p' && cameraState.targetPlayer) {
      // Convert player name to index using the player list
      const playerIndex = this._getPlayerIndex(cameraState.targetPlayer);
      if (playerIndex !== -1) {
        frame.tp = playerIndex;
      }
    }

    this.frames.push(frame);
    this.recordingDuration = timeOffset;
  }

  /**
   * Get recording state
   * @returns {boolean} Whether currently recording
   */
  isActive() {
    return this.isRecording;
  }

  /**
   * Get current recording data (without stopping)
   * @returns {Object} Current recording data
   */
  getData() {
    return {
      type: 'capture',
      sampleRate: this.sampleRate,
      duration: Math.round(this.recordingDuration),
      frames: [...this.frames],
    };
  }

  /**
   * Get frame count
   * @returns {number} Number of captured frames
   */
  getFrameCount() {
    return this.frames.length;
  }

  /**
   * Get recording duration in seconds
   * @returns {number} Duration in seconds
   */
  getDuration() {
    return this.recordingDuration / 1000;
  }

  /**
   * Clear all recorded data
   */
  clear() {
    this.frames = [];
    this.recordingDuration = 0;
    this.lastCaptureTime = -1;
    this.accumulatedTime = 0;
  }

  /**
   * Round a number to specified decimal places
   * @private
   */
  _round(value, decimals) {
    const factor = Math.pow(10, decimals);
    return Math.round(value * factor) / factor;
  }

  /**
   * Get player index from player name
   * @private
   * @param {string} playerName - Player name to find
   * @returns {number} Player index or -1 if not found
   */
  _getPlayerIndex(playerName) {
    if (!this.playerList || !playerName) return -1;

    const index = this.playerList.findIndex(p => p.name === playerName);
    return index;
  }
}
