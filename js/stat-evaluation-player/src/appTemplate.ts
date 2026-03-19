export function getAppTemplate(defaultCameraDistanceScale: number): string {
  return `
  <main class="shell">
    <section class="hero">
      <div>
        <p class="eyebrow">subtr-actor / stats replay viewer</p>
        <h1>Stat Evaluation Player</h1>
        <p class="lede">
          Compare stat modules against the in-replay camera view, switch to any
          player's camera profile, and scrub with the shared timeline plugin.
        </p>
      </div>
      <label class="file-picker">
        <span>Choose replay</span>
        <input id="replay-file" type="file" accept=".replay" />
      </label>
    </section>

    <section class="workspace">
      <div class="viewport-column">
        <div class="viewport-panel">
          <div id="viewport" class="viewport"></div>
          <div
            id="followed-player-overlay"
            class="followed-player-overlay"
            hidden
          ></div>
          <div id="empty-state" class="empty-state">
            Choose a replay to start the viewer.
          </div>
        </div>

        <section class="stats-panel">
          <div class="panel-heading">
            <div>
              <p class="panel-eyebrow">Module output</p>
              <h2>Per-player stats</h2>
            </div>
          </div>
          <div id="player-stats" class="player-stats-stack">
            Load a replay to see stats.
          </div>
        </section>
      </div>

      <aside class="sidebar">
        <section class="panel">
          <p class="panel-eyebrow">Camera</p>
          <h2>Replay camera</h2>
          <label>
            <span class="label">Camera profile</span>
            <select id="attached-player" disabled>
              <option value="">Free camera</option>
            </select>
          </label>
          <label>
            <span class="label">Follow distance</span>
            <input
              id="camera-distance"
              type="range"
              min="0.75"
              max="4"
              step="0.05"
              value="${defaultCameraDistanceScale}"
              disabled
            />
          </label>
          <strong id="camera-distance-readout" class="metric-readout">
            ${defaultCameraDistanceScale.toFixed(2)}x
          </strong>
          <label class="toggle">
            <input id="ball-cam" type="checkbox" disabled />
            <span>Ball cam</span>
          </label>
          <dl class="detail-grid">
            <div>
              <dt>Profile</dt>
              <dd id="camera-profile-readout">Free camera</dd>
            </div>
            <div>
              <dt>FOV</dt>
              <dd id="camera-fov-readout">--</dd>
            </div>
            <div>
              <dt>Height</dt>
              <dd id="camera-height-readout">--</dd>
            </div>
            <div>
              <dt>Pitch</dt>
              <dd id="camera-pitch-readout">--</dd>
            </div>
            <div>
              <dt>Distance</dt>
              <dd id="camera-base-distance-readout">--</dd>
            </div>
            <div>
              <dt>Stiffness</dt>
              <dd id="camera-stiffness-readout">--</dd>
            </div>
          </dl>
        </section>

        <section class="panel">
          <p class="panel-eyebrow">Modules</p>
          <h2>Overlay modules</h2>
          <p class="panel-copy">
            Toggle stat overlays independently while keeping the timeline and
            replay camera controls active.
          </p>
          <label class="toggle">
            <input id="show-followed-player-overlay" type="checkbox" />
            <span>Show followed player in viewport</span>
          </label>
          <div class="module-list" id="module-summary"></div>
          <div id="module-settings" class="module-settings" hidden></div>
        </section>

        <section class="panel">
          <p class="panel-eyebrow">Transport</p>
          <h2>Playback</h2>
          <div class="transport-row">
            <button id="toggle-playback" disabled>Play</button>
            <select id="playback-rate" disabled>
              <option value="0.25">0.25x</option>
              <option value="0.5">0.5x</option>
              <option value="1" selected>1.0x</option>
              <option value="1.5">1.5x</option>
              <option value="2">2.0x</option>
            </select>
          </div>
          <label class="toggle">
            <input id="skip-post-goal-transitions" type="checkbox" checked />
            <span>Skip post-goal resets</span>
          </label>
          <label class="toggle">
            <input id="skip-kickoffs" type="checkbox" />
            <span>Skip kickoffs</span>
          </label>
          <div class="detail-grid">
            <div>
              <dt>Time</dt>
              <dd id="time-readout">0.00s</dd>
            </div>
            <div>
              <dt>Frame</dt>
              <dd id="frame-readout">0</dd>
            </div>
            <div>
              <dt>Duration</dt>
              <dd id="duration-readout">0.00s</dd>
            </div>
            <div>
              <dt>Status</dt>
              <dd id="playback-status-readout">Stopped</dd>
            </div>
          </div>
        </section>

        <section class="panel">
          <p class="panel-eyebrow">Replay</p>
          <h2>Loaded file</h2>
          <dl class="detail-grid">
            <div>
              <dt>Status</dt>
              <dd id="status-readout">Waiting for file</dd>
            </div>
            <div>
              <dt>Players</dt>
              <dd id="players-readout">--</dd>
            </div>
            <div>
              <dt>Frames</dt>
              <dd id="frames-readout">--</dd>
            </div>
            <div>
              <dt>Timeline events</dt>
              <dd id="events-readout">--</dd>
            </div>
          </dl>
        </section>
      </aside>
    </section>
  </main>
`;
}
