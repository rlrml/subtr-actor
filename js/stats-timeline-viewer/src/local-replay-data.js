const UNIT_SCALE = 1
const DEFAULT_CAMERA_SETTINGS = {
  distance: 270,
  height: 100,
  pitch: -4,
  fov: 110,
}

function scaleVector(value) {
  return {
    x: value.x * UNIT_SCALE,
    y: value.y * UNIT_SCALE,
    z: value.z * UNIT_SCALE,
  }
}

function mapVectorToScene(value) {
  return {
    x: value.x,
    y: value.z,
    z: value.y,
  }
}

function playerIdToString(playerId) {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"]
  return `${kind}:${value}`
}

function normalizeVector(value) {
  const magnitude = Math.hypot(value.x, value.y, value.z)
  if (magnitude < 0.000001) {
    return null
  }

  return {
    x: value.x / magnitude,
    y: value.y / magnitude,
    z: value.z / magnitude,
  }
}

function normalizeQuaternion(raw) {
  const magnitude = Math.hypot(raw.x, raw.y, raw.z, raw.w)
  if (magnitude < 0.000001) {
    return null
  }

  return {
    x: raw.x / magnitude,
    y: raw.y / magnitude,
    z: raw.z / magnitude,
    w: raw.w / magnitude,
  }
}

function multiplyQuaternions(left, right) {
  return {
    w:
      left.w * right.w -
      left.x * right.x -
      left.y * right.y -
      left.z * right.z,
    x:
      left.w * right.x +
      left.x * right.w +
      left.y * right.z -
      left.z * right.y,
    y:
      left.w * right.y -
      left.x * right.z +
      left.y * right.w +
      left.z * right.x,
    z:
      left.w * right.z +
      left.x * right.y -
      left.y * right.x +
      left.z * right.w,
  }
}

function rotateVectorByQuaternion(vector, quaternion) {
  const rotated = multiplyQuaternions(
    multiplyQuaternions(quaternion, {
      x: vector.x,
      y: vector.y,
      z: vector.z,
      w: 0,
    }),
    {
      x: -quaternion.x,
      y: -quaternion.y,
      z: -quaternion.z,
      w: quaternion.w,
    },
  )

  return {
    x: rotated.x,
    y: rotated.y,
    z: rotated.z,
  }
}

function parseBallFrame(frame) {
  if (frame === "Empty") {
    return { position: null }
  }

  return {
    position: scaleVector(frame.Data.rigid_body.location),
  }
}

function parsePlayerFrame(frame) {
  if (frame === "Empty") {
    return {
      position: null,
      velocity: null,
      forward: null,
      up: null,
      boostAmount: 0,
      boostActive: false,
      jumpActive: false,
      dodgeActive: false,
    }
  }

  const rotation = normalizeQuaternion(frame.Data.rigid_body.rotation)
  const forwardRl = rotation
    ? rotateVectorByQuaternion({ x: 0, y: 0, z: 1 }, rotation)
    : null
  const upRl = rotation
    ? rotateVectorByQuaternion({ x: -1, y: 0, z: 0 }, rotation)
    : null

  return {
    position: scaleVector(frame.Data.rigid_body.location),
    velocity: scaleVector(frame.Data.rigid_body.linear_velocity),
    forward: forwardRl ? normalizeVector(mapVectorToScene(forwardRl)) : null,
    up: upRl ? normalizeVector(mapVectorToScene(upRl)) : null,
    boostAmount: frame.Data.boost_amount,
    boostActive: frame.Data.boost_active,
    jumpActive: frame.Data.jump_active,
    dodgeActive: frame.Data.dodge_active,
  }
}

function buildPlaybackFrames(raw) {
  const startTime = raw.frame_data.metadata_frames[0]?.time ?? 0
  return raw.frame_data.metadata_frames.map((frame) => ({
    time: frame.time - startTime,
    secondsRemaining: frame.seconds_remaining,
    gameState: frame.replicated_game_state_name,
  }))
}

function inferTeamSide(name, teamZeroNames, teamOneNames, firstFrame) {
  if (teamZeroNames.has(name)) {
    return true
  }

  if (teamOneNames.has(name)) {
    return false
  }

  if (firstFrame && firstFrame !== "Empty" && firstFrame.Data.is_team_0 !== undefined) {
    return firstFrame.Data.is_team_0
  }

  return true
}

function getStatEntries(stats) {
  if (!stats) {
    return []
  }

  if (stats instanceof Map) {
    return Array.from(stats.entries())
  }

  return Object.entries(stats)
}

function extractNumericSetting(entries, key) {
  const value = entries.find(([entryKey]) => entryKey === key)?.[1]
  return typeof value === "number" && Number.isFinite(value) ? value : undefined
}

function extractCameraSettings(playerInfo) {
  const entries = getStatEntries(playerInfo?.stats)
  return {
    fov: extractNumericSetting(entries, "CameraFOV") ?? DEFAULT_CAMERA_SETTINGS.fov,
    height:
      extractNumericSetting(entries, "CameraHeight") ??
      DEFAULT_CAMERA_SETTINGS.height,
    pitch:
      extractNumericSetting(entries, "CameraPitch") ?? DEFAULT_CAMERA_SETTINGS.pitch,
    distance:
      extractNumericSetting(entries, "CameraDistance") ??
      DEFAULT_CAMERA_SETTINGS.distance,
    stiffness:
      extractNumericSetting(entries, "CameraStiffness") ??
      DEFAULT_CAMERA_SETTINGS.stiffness,
    swivelSpeed:
      extractNumericSetting(entries, "CameraSwivelSpeed") ??
      DEFAULT_CAMERA_SETTINGS.swivelSpeed,
    transitionSpeed:
      extractNumericSetting(entries, "CameraTransitionSpeed") ??
      DEFAULT_CAMERA_SETTINGS.transitionSpeed,
  }
}

function indexReplayPlayers(raw) {
  const byId = new Map()
  const byName = new Map()

  for (const playerInfo of [...raw.meta.team_zero, ...raw.meta.team_one]) {
    byName.set(playerInfo.name, playerInfo)
    if (playerInfo.remote_id) {
      byId.set(playerIdToString(playerInfo.remote_id), playerInfo)
    }
  }

  return { byId, byName }
}

function buildPlayerTracks(raw) {
  const teamZeroNames = new Set(raw.meta.team_zero.map((player) => player.name))
  const teamOneNames = new Set(raw.meta.team_one.map((player) => player.name))
  const replayPlayers = indexReplayPlayers(raw)

  return raw.frame_data.players.map(([playerId, playerData]) => {
    const firstFrame = playerData.frames.find((frame) => frame !== "Empty")
    const playerIdString = playerIdToString(playerId)
    const name =
      firstFrame !== undefined && firstFrame.Data.player_name
        ? firstFrame.Data.player_name
        : playerIdString
    const replayPlayerInfo =
      replayPlayers.byId.get(playerIdString) ?? replayPlayers.byName.get(name)

    return {
      id: playerIdString,
      name,
      isTeamZero: inferTeamSide(name, teamZeroNames, teamOneNames, firstFrame),
      cameraSettings: extractCameraSettings(replayPlayerInfo),
      frames: playerData.frames.map(parsePlayerFrame),
    }
  })
}

export function normalizeReplayData(raw) {
  const frames = buildPlaybackFrames(raw)
  const players = buildPlayerTracks(raw)

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    frames,
    ballFrames: raw.frame_data.ball_data.frames.map(parseBallFrame),
    players,
    teamZeroNames: raw.meta.team_zero.map((player) => player.name),
    teamOneNames: raw.meta.team_one.map((player) => player.name),
  }
}

export function findFrameIndexAtTime(replay, time) {
  if (replay.frames.length === 0) {
    return 0
  }

  let low = 0
  let high = replay.frames.length - 1

  while (low <= high) {
    const middle = Math.floor((low + high) / 2)
    const middleTime = replay.frames[middle]?.time ?? 0

    if (middleTime < time) {
      low = middle + 1
    } else if (middleTime > time) {
      high = middle - 1
    } else {
      return middle
    }
  }

  return Math.max(0, low - 1)
}
