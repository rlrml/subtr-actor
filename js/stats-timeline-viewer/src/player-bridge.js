import playerScriptUrl from "../../example/src/player.js?url";

let dependenciesReady = false;

export async function createReplayPlayer(replayData, onTimeUpdate) {
  window.replayData = replayData;
  await ensureViewerDependencies();

  document.getElementById("player").replaceChildren();
  document.getElementById("player-container").style.display = "block";
  document.getElementById("details-watch").style.display = "block";

  const settings = window.Settings({
    "player.autoplay": false,
    "player.speed": 3,
    "cars.colors.simple": false,
    "cars.name.hide": false,
    "cars.bam.hide": false,
    "boost.pads.hide": true,
    "cars.boost.trail.hide": false,
    "cars.trails": false,
    "ball.trail": false,
    "trail.duration": 1,
  });

  window.ReplayPlayer({ settings, slave: null });
  const player = window.player;
  if (!player?.bus) {
    throw new Error("Replay player failed to initialize.");
  }

  if (onTimeUpdate) {
    player.bus.on("time-update", onTimeUpdate);
    player.bus.on("set-time", onTimeUpdate);
    onTimeUpdate(player.currentTime ?? 0);
  }
  return player;
}

async function ensureViewerDependencies() {
  if (dependenciesReady) {
    return;
  }

  await loadScript("https://cdnjs.cloudflare.com/ajax/libs/jquery/3.6.0/jquery.min.js");
  await loadScript("https://cdnjs.cloudflare.com/ajax/libs/three.js/r116/three.min.js");
  await loadOptionalScript(
    "https://unpkg.com/stats.js@0.17.0/build/stats.min.js",
    () =>
      function StatsStub() {
        return { begin() {}, end() {}, showPanel() {}, dom: document.createElement("div") };
      },
    "Stats",
  );
  await loadOptionalScript(
    "https://unpkg.com/hotkeys-js@3.8.1/dist/hotkeys.min.js",
    () => function hotkeysStub() {},
    "hotkeys",
  );

  installCookieHelpers();
  ensureParticleShaders();
  await loadScript(playerScriptUrl);
  dependenciesReady = true;
}

function installCookieHelpers() {
  window.readCookie =
    window.readCookie ||
    function readCookie(name) {
      const prefix = `${name}=`;
      for (const value of document.cookie.split(";")) {
        const trimmed = value.trim();
        if (trimmed.startsWith(prefix)) {
          return trimmed.slice(prefix.length);
        }
      }
      return null;
    };

  window.createCookie =
    window.createCookie ||
    function createCookie(name, value, days) {
      let expires = "";
      if (days) {
        const date = new Date();
        date.setTime(date.getTime() + days * 24 * 60 * 60 * 1000);
        expires = `; expires=${date.toUTCString()}`;
      }
      document.cookie = `${name}=${value}${expires}; path=/`;
    };
}

function ensureParticleShaders() {
  if (!document.getElementById("particle-vertex-shader")) {
    const vertexShader = document.createElement("script");
    vertexShader.id = "particle-vertex-shader";
    vertexShader.type = "x-shader/x-vertex";
    vertexShader.textContent = `
      attribute float size;
      attribute float age;
      varying float vAge;
      void main() {
          vAge = age;
          vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
          gl_PointSize = size * (300.0 / -mvPosition.z);
          gl_Position = projectionMatrix * mvPosition;
      }
    `;
    document.head.appendChild(vertexShader);
  }

  if (!document.getElementById("particle-fragment-shader")) {
    const fragmentShader = document.createElement("script");
    fragmentShader.id = "particle-fragment-shader";
    fragmentShader.type = "x-shader/x-fragment";
    fragmentShader.textContent = `
      uniform sampler2D texture;
      varying float vAge;
      void main() {
          gl_FragColor = vec4(color, vAge) * texture2D(texture, gl_PointCoord);
      }
    `;
    document.head.appendChild(fragmentShader);
  }

  if (window.THREE?.TextureLoader) {
    const canvas = document.createElement("canvas");
    canvas.width = 64;
    canvas.height = 64;
    const context = canvas.getContext("2d");
    const gradient = context.createRadialGradient(32, 32, 0, 32, 32, 30);
    gradient.addColorStop(0, "rgba(255,255,255,1)");
    gradient.addColorStop(0.5, "rgba(255,255,255,0.5)");
    gradient.addColorStop(1, "rgba(255,255,255,0)");
    context.fillStyle = gradient;
    context.fillRect(0, 0, 64, 64);

    const texture = new THREE.CanvasTexture(canvas);
    texture.needsUpdate = true;
    const originalLoad = THREE.TextureLoader.prototype.load;
    THREE.TextureLoader.prototype.load = function loadTexture(url, onLoad, onProgress, onError) {
      if (
        url === "/static/textures/solid-particle.png" ||
        url === "/static/textures/lensflare0_alpha.png"
      ) {
        if (onLoad) {
          onLoad(texture);
        }
        return texture;
      }
      return originalLoad.call(this, url, onLoad, onProgress, onError);
    };
  }
}

function loadScript(src) {
  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = src;
    script.onload = () => resolve();
    script.onerror = (error) => reject(new Error(`Failed to load script ${src}: ${error}`));
    document.head.appendChild(script);
  });
}

async function loadOptionalScript(src, fallbackFactory, globalName) {
  try {
    await loadScript(src);
  } catch (error) {
    console.warn(error);
    if (!window[globalName]) {
      window[globalName] = fallbackFactory();
    }
  }
}
