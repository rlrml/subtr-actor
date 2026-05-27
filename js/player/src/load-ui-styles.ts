const STYLE_ID = "subtr-actor-replay-load-overlay-styles";

export function ensureReplayLoadOverlayStyles(): void {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = `
    .sap-load-overlay {
      position: absolute;
      inset: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 20px;
      background:
        radial-gradient(circle at top, rgba(255,255,255,0.12), transparent 50%),
        rgba(10, 15, 26, 0.72);
      backdrop-filter: blur(8px);
      z-index: 40;
      pointer-events: none;
    }

    .sap-load-overlay__panel {
      width: min(360px, 100%);
      padding: 18px 20px;
      border: 1px solid rgba(255,255,255,0.14);
      border-radius: 16px;
      background: rgba(8, 12, 20, 0.88);
      box-shadow: 0 20px 60px rgba(0,0,0,0.35);
      color: #f5f7fb;
      font: 500 14px/1.4 "IBM Plex Sans", "Avenir Next", sans-serif;
      letter-spacing: 0.01em;
    }

    .sap-load-overlay__title {
      margin: 0 0 10px;
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: rgba(255,255,255,0.64);
    }

    .sap-load-overlay__status {
      margin: 0 0 12px;
      font-size: 15px;
      color: #ffffff;
    }

    .sap-load-overlay__bar {
      overflow: hidden;
      height: 8px;
      border-radius: 999px;
      background: rgba(255,255,255,0.12);
    }

    .sap-load-overlay__fill {
      width: 0%;
      height: 100%;
      border-radius: inherit;
      background: linear-gradient(90deg, #58c4dd 0%, #f4b860 100%);
      transition: width 120ms linear;
    }

    .sap-load-overlay__meta {
      margin-top: 10px;
      font-size: 12px;
      color: rgba(255,255,255,0.6);
    }

    .sap-load-overlay__panel[data-state="error"] .sap-load-overlay__fill {
      background: linear-gradient(90deg, #ff6b6b 0%, #ff9b6b 100%);
      width: 100% !important;
    }
  `;
  document.head.append(style);
}
