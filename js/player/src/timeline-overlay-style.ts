const STYLE_ID = "subtr-actor-timeline-overlay-styles";

export function ensureTimelineOverlayStyles(): void {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = `
    .sap-tl-root {
      position: absolute;
      inset: 0;
      z-index: 4;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Segoe UI", Roboto, sans-serif;
    }

    .sap-tl-shell {
      --sap-tl-thumb-size: 1.35rem;
      --sap-tl-track-height: 0.6rem;
      --sap-tl-gutter-width: 2.25rem;
      --sap-tl-gutter-gap: 0.55rem;
      --sap-tl-marker-offset: 1.05rem;
      position: absolute;
      left: 0.8rem;
      right: 0.8rem;
      bottom: 0.9rem;
      padding: 0.75rem 0.9rem 0.9rem;
      border: 1px solid rgba(180, 205, 226, 0.18);
      border-radius: 1.05rem;
      background:
        linear-gradient(180deg, rgba(13, 20, 28, 0.92), rgba(7, 12, 18, 0.96));
      box-shadow: 0 18px 42px rgba(0, 0, 0, 0.28);
      backdrop-filter: blur(12px);
      pointer-events: auto;
    }

    .sap-tl-shell::before {
      content: "";
      position: absolute;
      inset: 0;
      border-radius: inherit;
      background:
        linear-gradient(90deg, rgba(60, 134, 255, 0.18), transparent 28%, transparent 72%, rgba(242, 138, 37, 0.16));
      pointer-events: none;
    }

    .sap-tl-topline {
      position: relative;
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-left: calc(var(--sap-tl-gutter-width) + var(--sap-tl-gutter-gap));
      margin-bottom: 0.55rem;
      color: #f5fbff;
      font-size: 0.82rem;
      font-weight: 600;
      font-variant-numeric: tabular-nums;
      gap: 0.85rem;
    }

    .sap-tl-primary {
      display: flex;
      align-items: center;
      gap: 0.65rem;
      min-width: 0;
    }

    .sap-tl-toggle {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 0.4rem;
      min-width: 4.9rem;
      padding: 0.42rem 0.72rem;
      border: 1px solid rgba(184, 214, 236, 0.24);
      border-radius: 999px;
      background: rgba(18, 30, 42, 0.92);
      color: #f5fbff;
      font: inherit;
      font-size: 0.76rem;
      font-weight: 700;
      letter-spacing: 0.02em;
      cursor: pointer;
      transition:
        transform 140ms ease,
        border-color 140ms ease,
        background 140ms ease;
    }

    .sap-tl-track-toggle {
      width: 2.15rem;
      min-width: 2.15rem;
      min-height: 2.15rem;
      padding: 0;
      gap: 0;
    }

    .sap-tl-toggle-label {
      display: none;
      min-width: 0;
    }

    .sap-tl-toggle:hover {
      border-color: rgba(184, 214, 236, 0.4);
      background: rgba(28, 45, 61, 0.96);
      transform: translateY(-1px);
    }

    .sap-tl-toggle:focus-visible {
      outline: 2px solid rgba(123, 180, 255, 0.9);
      outline-offset: 2px;
    }

    .sap-tl-toggle-icon {
      width: 0.85rem;
      text-align: center;
      font-size: 0.7rem;
      line-height: 1;
    }

    .sap-tl-current {
      color: #f5fbff;
    }

    .sap-tl-remaining {
      color: #b8c9d9;
    }

    .sap-tl-track-wrap {
      position: relative;
      display: grid;
      grid-template-columns: var(--sap-tl-gutter-width) minmax(0, 1fr);
      column-gap: var(--sap-tl-gutter-gap);
      row-gap: 0;
      align-items: center;
    }

    .sap-tl-ranges {
      grid-column: 1 / -1;
      display: flex;
      flex-direction: column;
      gap: 0.34rem;
      margin-bottom: 0;
    }

    .sap-tl-event-lanes {
      grid-column: 1 / -1;
      display: flex;
      flex-direction: column;
      gap: 0.34rem;
      margin-bottom: 0;
    }

    .sap-tl-event-lane {
      position: relative;
      display: grid;
      grid-template-columns: var(--sap-tl-gutter-width) minmax(0, 1fr);
      column-gap: var(--sap-tl-gutter-gap);
      align-items: center;
    }

    .sap-tl-event-lane-track {
      position: relative;
      grid-column: 2;
      height: 1.05rem;
      margin: 0 calc(var(--sap-tl-thumb-size) / 2);
      box-sizing: border-box;
      border-radius: 999px;
      background: rgba(255, 255, 255, 0.045);
      box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.07);
    }

    .sap-tl-event-lane-label {
      display: block;
      max-width: 100%;
      padding: 0.08rem 0.38rem;
      border: 1px solid rgba(184, 214, 236, 0.18);
      border-radius: 999px;
      background: rgba(10, 16, 23, 0.82);
      color: #c8d7e4;
      font-size: 0.54rem;
      font-weight: 800;
      letter-spacing: 0.04em;
      line-height: 1.2;
      text-transform: uppercase;
      backdrop-filter: blur(6px);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .sap-tl-range-lane {
      position: relative;
      display: grid;
      grid-template-columns: var(--sap-tl-gutter-width) minmax(0, 1fr);
      column-gap: var(--sap-tl-gutter-gap);
      align-items: center;
    }

    .sap-tl-range-lane-track {
      position: relative;
      grid-column: 2;
      height: var(--sap-tl-track-height);
      margin: 0 calc(var(--sap-tl-thumb-size) / 2);
      box-sizing: border-box;
      border-radius: 999px;
      background: rgba(255, 255, 255, 0.06);
      box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
      overflow: hidden;
    }

    .sap-tl-range-lane-label {
      display: block;
      max-width: 100%;
      padding: 0.08rem 0.38rem;
      border: 1px solid rgba(184, 214, 236, 0.18);
      border-radius: 999px;
      background: rgba(10, 16, 23, 0.82);
      color: #c8d7e4;
      font-size: 0.54rem;
      font-weight: 800;
      letter-spacing: 0.04em;
      line-height: 1.2;
      text-transform: uppercase;
      backdrop-filter: blur(6px);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .sap-tl-event-lane[data-label]::after,
    .sap-tl-range-lane[data-label]::after {
      content: attr(data-label);
      position: absolute;
      left: calc(var(--sap-tl-gutter-width) + var(--sap-tl-gutter-gap) + calc(var(--sap-tl-thumb-size) / 2));
      bottom: calc(100% + 0.28rem);
      z-index: 8;
      max-width: min(22rem, calc(100% - var(--sap-tl-gutter-width) - var(--sap-tl-gutter-gap)));
      padding: 0.28rem 0.48rem;
      border: 1px solid rgba(184, 214, 236, 0.24);
      border-radius: 0.4rem;
      background: rgba(7, 12, 18, 0.96);
      color: #f5fbff;
      box-shadow: 0 10px 24px rgba(0, 0, 0, 0.34);
      font-size: 0.68rem;
      font-weight: 800;
      line-height: 1.2;
      opacity: 0;
      overflow: hidden;
      pointer-events: none;
      text-overflow: ellipsis;
      transform: translateY(0.14rem);
      transition:
        opacity 120ms ease,
        transform 120ms ease;
      white-space: nowrap;
    }

    .sap-tl-event-lane[data-label]:hover::after,
    .sap-tl-event-lane[data-label]:focus-within::after,
    .sap-tl-range-lane[data-label]:hover::after,
    .sap-tl-range-lane[data-label]:focus-within::after {
      opacity: 1;
      transform: translateY(0);
    }

    .sap-tl-range-segment {
      position: absolute;
      top: 0;
      bottom: 0;
      min-width: 2px;
      border-radius: 999px;
      opacity: 0.62;
      transition:
        opacity 120ms ease,
        filter 120ms ease,
        transform 120ms ease;
    }

    .sap-tl-range-segment[data-active="true"] {
      opacity: 0.92;
      filter: brightness(1.12);
      transform: scaleY(1.06);
    }

    .sap-tl-range-playhead,
    .sap-tl-event-playhead {
      position: absolute;
      top: -0.14rem;
      bottom: -0.14rem;
      width: 1px;
      transform: translateX(-50%);
      border-radius: 999px;
      background: rgba(245, 251, 255, 0.74);
      box-shadow: 0 0 0 1px rgba(6, 12, 18, 0.45);
      opacity: 0.9;
      pointer-events: none;
      z-index: 3;
    }

    .sap-tl-event-playhead {
      top: -0.08rem;
      bottom: -0.08rem;
    }

    .sap-tl-track-rail {
      position: relative;
      grid-column: 2;
      min-width: 0;
      min-height: var(--sap-tl-thumb-size);
      margin-top: 0.58rem;
    }

    .sap-tl-main-rail {
      position: absolute;
      left: calc(var(--sap-tl-thumb-size) / 2);
      right: calc(var(--sap-tl-thumb-size) / 2);
      top: 50%;
      height: var(--sap-tl-track-height);
      border-radius: 999px;
      border: 1px solid rgba(255, 255, 255, 0.12);
      background:
        linear-gradient(90deg, rgba(60, 134, 255, 0.42), rgba(103, 179, 255, 0.58) 45%, rgba(242, 138, 37, 0.58));
      box-shadow: inset 0 0 0 999px rgba(5, 10, 15, 0.4);
      transform: translateY(-50%);
      pointer-events: none;
      z-index: 0;
    }

    .sap-tl-range {
      position: relative;
      z-index: 2;
      width: 100%;
      height: var(--sap-tl-thumb-size);
      margin: 0;
      appearance: none;
      background: transparent;
      cursor: pointer;
    }

    .sap-tl-range:focus {
      outline: none;
    }

    .sap-tl-range::-webkit-slider-runnable-track {
      height: var(--sap-tl-track-height);
      border-radius: 999px;
      border: 0;
      background: transparent;
      box-shadow: none;
    }

    .sap-tl-range::-moz-range-track {
      height: var(--sap-tl-track-height);
      border-radius: 999px;
      border: 0;
      background: transparent;
      box-shadow: none;
    }

    .sap-tl-range::-webkit-slider-thumb {
      appearance: none;
      margin-top: -0.38rem;
      width: var(--sap-tl-thumb-size);
      height: var(--sap-tl-thumb-size);
      border: 0;
      border-radius: 50%;
      background:
        radial-gradient(circle at 35% 35%, #ffffff 0%, #d8ebff 28%, #7bb4ff 55%, #27456d 100%);
      box-shadow: 0 8px 22px rgba(0, 0, 0, 0.34);
    }

    .sap-tl-range::-moz-range-thumb {
      width: var(--sap-tl-thumb-size);
      height: var(--sap-tl-thumb-size);
      border: 0;
      border-radius: 50%;
      background:
        radial-gradient(circle at 35% 35%, #ffffff 0%, #d8ebff 28%, #7bb4ff 55%, #27456d 100%);
      box-shadow: 0 8px 22px rgba(0, 0, 0, 0.34);
    }

    .sap-tl-shell[data-scrubbing="true"] .sap-tl-range::-webkit-slider-thumb,
    .sap-tl-shell[data-scrubbing="true"] .sap-tl-range::-moz-range-thumb {
      background:
        radial-gradient(circle at 35% 35%, #ffffff 0%, #ffe5c5 32%, #ffad47 58%, #7b3d00 100%);
      transform: scale(1.05);
    }

    .sap-tl-markers {
      position: absolute;
      left: calc(var(--sap-tl-thumb-size) / 2);
      right: calc(var(--sap-tl-thumb-size) / 2);
      top: calc(-1 * var(--sap-tl-marker-offset));
      height: 1rem;
      pointer-events: none;
      z-index: 1;
    }

    .sap-tl-event-lane .sap-tl-markers {
      position: relative;
      left: auto;
      right: auto;
      top: auto;
      height: 100%;
    }

    .sap-tl-event-lane .sap-tl-marker {
      top: 50%;
      transform: translate(-50%, -50%);
    }

    .sap-tl-event-lane .sap-tl-marker::before {
      display: none;
    }

    .sap-tl-event-lane .sap-tl-marker[data-active="true"] {
      transform: translate(-50%, -50%) scale(1.16);
    }

    .sap-tl-marker {
      position: absolute;
      top: 0;
      transform: translateX(-50%);
      width: 0.95rem;
      height: 0.95rem;
      padding: 0;
      border: 0;
      border-radius: 999px;
      background: rgba(12, 18, 24, 0.96);
      color: #f5fbff;
      font-size: 0.52rem;
      font-weight: 800;
      line-height: 1;
      box-shadow: 0 4px 14px rgba(0, 0, 0, 0.3);
      pointer-events: auto;
      cursor: pointer;
    }

    .sap-tl-marker::before {
      content: "";
      position: absolute;
      left: 50%;
      top: 0.85rem;
      width: 2px;
      height: 0.55rem;
      transform: translateX(-50%);
      background: currentColor;
      opacity: 0.7;
    }

    .sap-tl-marker:hover {
      filter: brightness(1.08);
    }

    .sap-tl-marker[data-passed="true"] {
      opacity: 0.9;
    }

    .sap-tl-marker[data-active="true"] {
      transform: translateX(-50%) scale(1.16);
      opacity: 1;
      box-shadow: 0 6px 18px rgba(0, 0, 0, 0.38);
    }

    @media (max-width: 720px) {
      .sap-tl-shell {
        --sap-tl-gutter-width: 4rem;
        --sap-tl-gutter-gap: 0.55rem;
        bottom: 0.6rem;
        left: 0.5rem;
        right: 0.5rem;
        padding: 0.65rem 0.7rem 0.75rem;
      }

      .sap-tl-topline {
        font-size: 0.72rem;
      }
    }
  `;
  document.head.append(style);
}
