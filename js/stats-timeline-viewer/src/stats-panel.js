import { buildPlayerNameMap } from "./replay-adapter.js";

export class StatsPanel {
  constructor(container) {
    this.container = container;
    this.timeline = null;
    this.lastFrameIndex = -1;
    this.nameById = new Map();
  }

  setTimeline(timeline) {
    this.timeline = timeline;
    this.nameById = buildPlayerNameMap(timeline.replay_meta ?? {});
    this.lastFrameIndex = -1;
  }

  updateTime(time) {
    if (!this.timeline) {
      return;
    }

    const frameIndex = findFrameIndexAtOrBefore(this.timeline.frames, time);
    if (frameIndex === this.lastFrameIndex || frameIndex < 0) {
      return;
    }

    this.lastFrameIndex = frameIndex;
    const frame = this.timeline.frames[frameIndex];
    const events = recentEventsForTime(this.timeline.timeline_events ?? [], time, 8);
    this.container.innerHTML = renderStatsPanel(frame, events, time, this.nameById);
  }
}

function renderStatsPanel(frame, events, time, nameById) {
  return `
    <div class="stats-header">
      <div>
        <p class="stats-eyebrow">Live Snapshot</p>
        <h2>${formatClock(time)}</h2>
      </div>
      <div class="status-pill ${frame.is_live_play ? "is-live" : "is-frozen"}">
        ${frame.is_live_play ? "Live Play" : "Non-Live"}
      </div>
    </div>
    <div class="summary-grid">
      ${renderSummaryCell("Frame", frame.frame_number)}
      ${renderSummaryCell("Seconds Left", frame.seconds_remaining ?? "n/a")}
      ${renderSummaryCell("Game State", frame.game_state ?? "n/a")}
      ${renderSummaryCell(
        "Score",
        `${frame.team_zero.core.goals} - ${frame.team_one.core.goals}`,
      )}
    </div>
    <section class="stats-block">
      <h3>Recent Events</h3>
      <div class="event-list">
        ${
          events.length === 0
            ? '<p class="empty-state">No events yet at this time.</p>'
            : events
                .map((event) => {
                  const playerName = event.player_id
                    ? nameById.get(JSON.stringify(event.player_id)) ?? "Unknown Player"
                    : "System";
                  return `
                    <div class="event-row">
                      <span class="event-kind">${event.kind}</span>
                      <span class="event-player">${playerName}</span>
                      <span class="event-time">${formatClock(event.time)}</span>
                    </div>
                  `;
                })
                .join("")
        }
      </div>
    </section>
    <section class="stats-block">
      <h3>Possession</h3>
      ${renderMetricGrid(frame.possession)}
    </section>
    <section class="stats-block team-blue">
      <h3>Blue Team</h3>
      ${renderTeamSnapshot(frame.team_zero)}
    </section>
    <section class="stats-block team-orange">
      <h3>Orange Team</h3>
      ${renderTeamSnapshot(frame.team_one)}
    </section>
    <section class="stats-block">
      <h3>Players</h3>
      <div class="player-list">
        ${frame.players.map((player) => renderPlayerSnapshot(player)).join("")}
      </div>
    </section>
  `;
}

function renderSummaryCell(label, value) {
  return `
    <div class="summary-cell">
      <div class="summary-label">${label}</div>
      <div class="summary-value">${value}</div>
    </div>
  `;
}

function renderTeamSnapshot(team) {
  return `
    ${renderNamedMetricGroup("Core", team.core)}
    ${renderNamedMetricGroup("Boost", team.boost)}
    ${renderNamedMetricGroup("Movement", team.movement)}
    ${renderNamedMetricGroup("Powerslide", team.powerslide)}
    ${renderNamedMetricGroup("Demo", team.demo)}
  `;
}

function renderPlayerSnapshot(player) {
  return `
    <article class="player-card ${player.is_team_0 ? "team-blue" : "team-orange"}">
      <header class="player-card-header">
        <div>
          <h4>${escapeHtml(player.name)}</h4>
          <p>${player.is_team_0 ? "Blue" : "Orange"} · ${escapeHtml(JSON.stringify(player.player_id))}</p>
        </div>
      </header>
      ${renderNamedMetricGroup("Core", player.core)}
      ${renderNamedMetricGroup("Boost", player.boost)}
      ${renderNamedMetricGroup("Movement", player.movement)}
      ${renderNamedMetricGroup("Positioning", player.positioning)}
      ${renderNamedMetricGroup("Powerslide", player.powerslide)}
      ${renderNamedMetricGroup("Demo", player.demo)}
    </article>
  `;
}

function renderNamedMetricGroup(title, stats) {
  return `
    <section class="metric-group">
      <h5>${title}</h5>
      ${renderMetricGrid(stats)}
    </section>
  `;
}

function renderMetricGrid(stats) {
  const rows = Object.entries(stats)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(
      ([key, value]) => `
        <div class="metric-row">
          <span class="metric-label">${formatKey(key)}</span>
          <span class="metric-value">${formatValue(value)}</span>
        </div>
      `,
    )
    .join("");

  return `<div class="metric-grid">${rows}</div>`;
}

function findFrameIndexAtOrBefore(frames, time) {
  let low = 0;
  let high = frames.length - 1;
  let result = -1;

  while (low <= high) {
    const middle = Math.floor((low + high) / 2);
    if (frames[middle].time <= time) {
      result = middle;
      low = middle + 1;
    } else {
      high = middle - 1;
    }
  }

  return result >= 0 ? result : 0;
}

function recentEventsForTime(events, time, limit) {
  const recent = [];
  for (let index = events.length - 1; index >= 0 && recent.length < limit; index -= 1) {
    if (events[index].time <= time) {
      recent.push(events[index]);
    }
  }
  return recent.reverse();
}

function formatKey(key) {
  return key
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatValue(value) {
  if (typeof value === "number") {
    if (Number.isInteger(value)) {
      return value.toString();
    }
    return value.toFixed(Math.abs(value) >= 100 ? 1 : 2).replace(/\.?0+$/, "");
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return escapeHtml(String(value));
}

function formatClock(time) {
  const minutes = Math.floor(time / 60);
  const seconds = Math.floor(time % 60);
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
