import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const jsDir = path.resolve(packageDir, "..");

function run(command, args, cwd) {
  execFileSync(command, args, {
    cwd,
    encoding: "utf8",
    stdio: "inherit",
  });
}

async function packTarball(cwd, packDir) {
  const packOutput = execFileSync("npm", ["pack", "--json", "--pack-destination", packDir], {
    cwd,
    encoding: "utf8",
  });
  const [{ filename }] = JSON.parse(packOutput);
  return path.join(packDir, filename);
}

async function main() {
  const sourcePackage = JSON.parse(
    await readFile(path.resolve(packageDir, "package.json"), "utf8"),
  );
  const playerPackage = JSON.parse(
    await readFile(path.resolve(jsDir, "player", "package.json"), "utf8"),
  );
  const scratchDir = await mkdtemp(path.join(os.tmpdir(), "subtr-actor-stats-player-smoke-"));
  let playerPublishDir = null;
  let statsPublishDir = null;

  try {
    run("npm", ["--prefix", jsDir, "run", "build"], packageDir);
    run("npm", ["run", "build"], path.resolve(jsDir, "player"));
    run("npm", ["run", "build"], packageDir);

    const packDir = path.join(scratchDir, "pack");
    const consumerDir = path.join(scratchDir, "consumer");
    const sourceDir = path.join(consumerDir, "src");

    await mkdir(packDir, { recursive: true });
    await mkdir(sourceDir, { recursive: true });

    const bindingsTarballPath = await packTarball(path.resolve(jsDir, "pkg"), packDir);

    playerPublishDir = execFileSync("npm", ["run", "--silent", "prepare:package"], {
      cwd: path.resolve(jsDir, "player"),
      encoding: "utf8",
    }).trim();
    const playerTarballPath = await packTarball(playerPublishDir, packDir);

    statsPublishDir = execFileSync("npm", ["run", "--silent", "prepare:package"], {
      cwd: packageDir,
      encoding: "utf8",
    }).trim();
    const statsTarballPath = await packTarball(statsPublishDir, packDir);

    await writeFile(
      path.join(consumerDir, "package.json"),
      JSON.stringify(
        {
          name: "subtr-actor-stats-player-smoke-consumer",
          private: true,
          type: "module",
          scripts: {
            check: "tsc --noEmit",
            build: "vite build",
          },
          dependencies: {
            "@rlrml/subtr-actor": `file:${path.relative(consumerDir, bindingsTarballPath)}`,
            [playerPackage.name]: `file:${path.relative(consumerDir, playerTarballPath)}`,
            [sourcePackage.name]: `file:${path.relative(consumerDir, statsTarballPath)}`,
            three: "^0.180.0",
          },
          devDependencies: {
            typescript: "^5.9.2",
            vite: "^7.3.1",
          },
        },
        null,
        2,
      ),
    );

    await writeFile(
      path.join(consumerDir, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            target: "ES2022",
            module: "ESNext",
            moduleResolution: "Bundler",
            lib: ["DOM", "ES2022"],
            strict: true,
            skipLibCheck: true,
          },
          include: ["src"],
        },
        null,
        2,
      ),
    );

    await writeFile(
      path.join(consumerDir, "index.html"),
      [
        "<!doctype html>",
        "<html>",
        "  <body>",
        '    <div id="app"></div>',
        '    <script type="module" src="/src/main.ts"></script>',
        "  </body>",
        "</html>",
        "",
      ].join("\n"),
    );

    await writeFile(
      path.join(sourceDir, "main.ts"),
      [
        "import {",
        "  buildTimeInZoneTimelineRanges,",
        "  createStatsFrameLookup,",
        "  mountStatEvaluationPlayer,",
        "  mountStatsReport,",
        "  type StatEvaluationPlayerHandle,",
        "  type StatsTimeline,",
        `} from "${sourcePackage.name}";`,
        "",
        'const root = document.getElementById("app");',
        "if (!(root instanceof HTMLElement)) {",
        '  throw new Error("Missing app root");',
        "}",
        "",
        "const emptyTimeline: StatsTimeline = {",
        "  config: {",
        "    most_back_forward_threshold_y: 0,",
        "    level_ball_depth_margin: 0,",
        "    pressure_neutral_zone_half_width_y: 0,",
        "    rotation_role_depth_margin: 0,",
        "    rotation_first_man_ambiguity_margin: 0,",
        "    rotation_first_man_debounce_seconds: 0,",
        "    rush_max_start_y: 0,",
        "    rush_attack_support_distance_y: 0,",
        "    rush_defender_distance_y: 0,",
        "    rush_min_possession_retained_seconds: 0,",
        "    aerial_goal_min_ball_z: 0,",
        "    high_aerial_goal_min_ball_z: 0,",
        "    long_distance_goal_max_attacking_y: 0,",
        "    own_half_goal_max_attacking_y: 0,",
        "    empty_net_min_defender_y_margin: 0,",
        "    empty_net_min_defender_distance: 0,",
        "    empty_net_max_touch_attacking_y: 0,",
        "    flick_goal_max_event_to_goal_seconds: 0,",
        "    double_tap_goal_max_event_to_goal_seconds: 0,",
        "    one_timer_goal_max_event_to_goal_seconds: 0,",
        "    air_dribble_goal_max_end_to_goal_seconds: 0,",
        "    flip_reset_goal_max_event_to_goal_seconds: 0,",
        "    half_volley_max_bounce_to_touch_seconds: 0,",
        "    half_volley_min_ball_speed: 0,",
        "    half_volley_goal_max_touch_to_goal_seconds: 0,",
        "    half_volley_goal_min_goal_alignment: 0,",
        "  },",
        "  replay_meta: {",
        "    team_zero: [],",
        "    team_one: [],",
        "    all_headers: [],",
        "  },",
        "  events: {",
        "    timeline: [],",
        "    core_player: [],",
        "    core_team: [],",
        "    possession: [],",
        "    pressure: [],",
        "    movement: [],",
        "    positioning: [],",
        "    rotation_player: [],",
        "    rotation_team: [],",
        "    mechanics: [],",
        "    goal_context: [],",
        "    backboard: [],",
        "    ceiling_shot: [],",
        "    wall_aerial: [],",
        "    wall_aerial_shot: [],",
        "    center: [],",
        "    flick: [],",
        "    musty_flick: [],",
        "    dodge_reset: [],",
        "    double_tap: [],",
        "    fifty_fifty: [],",
        "    one_timer: [],",
        "    pass: [],",
        "    pass_last_completed: [],",
        "    ball_carry: [],",
        "    goal_tags: [],",
        "    rush: [],",
        "    speed_flip: [],",
        "    half_flip: [],",
        "    half_volley: [],",
        "    wavedash: [],",
        "    whiff: [],",
        "    powerslide: [],",
        "    touch: [],",
        "    touch_ball_movement: [],",
        "    touch_last_touch: [],",
        "    boost_pickups: [],",
        "    boost_ledger: [],",
        "    boost_state: [],",
        "    bump: [],",
        "  },",
        "  frames: [],",
        "};",
        "createStatsFrameLookup(emptyTimeline);",
        "buildTimeInZoneTimelineRanges(emptyTimeline);",
        'const reportRoot = document.createElement("div");',
        "mountStatsReport(reportRoot, {",
        "  initialData: {",
        '    fileName: "empty.replay",',
        "    replayUrl: null,",
        "    statsTimeline: emptyTimeline,",
        "  },",
        "});",
        "const handle: StatEvaluationPlayerHandle = mountStatEvaluationPlayer(root);",
        "handle.destroy();",
        "",
      ].join("\n"),
    );

    run("npm", ["install"], consumerDir);
    run("npm", ["run", "check"], consumerDir);
    run("npm", ["run", "build"], consumerDir);
  } finally {
    if (playerPublishDir) {
      await rm(playerPublishDir, { force: true, recursive: true });
    }
    if (statsPublishDir) {
      await rm(statsPublishDir, { force: true, recursive: true });
    }
    await rm(scratchDir, { force: true, recursive: true });
  }
}

await main();
