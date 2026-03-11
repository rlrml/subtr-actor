import * as THREE from "three"
import { createReplayScene } from "./local-scene.js"
import { findFrameIndexAtTime } from "./local-replay-data.js"

const UP_OFFSET = 0.25
const BALL_RADIUS = 1.82
const CAMERA_DISTANCE_SCALE = 0.01
const CAMERA_HEIGHT_SCALE = 0.01
const ATTACHED_DISTANCE_MULTIPLIER = 1.8
const THIRD_PERSON_DISTANCE_MULTIPLIER = 2.7
const ATTACHED_HEIGHT_MULTIPLIER = 1.2
const THIRD_PERSON_HEIGHT_MULTIPLIER = 1.6
const CAMERA_SMOOTHING = 0.18
const MIN_CAMERA_HEIGHT = 0.9
const DEFAULT_FORWARD = new THREE.Vector3(0, 0, 1)
const DEFAULT_UP = new THREE.Vector3(0, 1, 0)

export class ReplayPlayer extends EventTarget {
  constructor(container, replay, options = {}) {
    super()
    this.container = container
    this.replay = replay
    this.options = options
    this.sceneState = createReplayScene(container, replay)
    this.animationFrameId = null
    this.lastTickTime = 0
    this.playing = false
    this.speed = 1
    this.currentTime = 0
    this.cameraMode = options.initialCameraMode ?? "overview"
    this.trackedPlayerId = options.initialTrackedPlayerId ?? replay.players[0]?.id ?? null
    this.ballCamEnabled = options.initialBallCamEnabled ?? false
    this.desiredCameraPosition = new THREE.Vector3()
    this.desiredLookTarget = new THREE.Vector3()
    this.boundResize = () => this.sceneState.resize()

    window.addEventListener("resize", this.boundResize)
    this.render()

    if (options.autoplay) {
      this.play()
    }
  }

  play() {
    if (this.playing) {
      return
    }

    this.playing = true
    this.lastTickTime = performance.now()
    this.tick()
    this.emitChange()
  }

  pause() {
    this.playing = false
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId)
      this.animationFrameId = null
    }
    this.emitChange()
  }

  togglePlayback() {
    if (this.playing) {
      this.pause()
    } else {
      this.play()
    }
  }

  setPlaybackRate(speed) {
    this.speed = Math.max(0.1, speed)
    this.emitChange()
  }

  setCameraMode(mode) {
    this.cameraMode = mode
    this.render()
    this.emitChange()
  }

  setTrackedPlayer(playerId) {
    this.trackedPlayerId = playerId
    this.render()
    this.emitChange()
  }

  setBallCamEnabled(enabled) {
    this.ballCamEnabled = enabled
    this.render()
    this.emitChange()
  }

  seek(time) {
    this.currentTime = THREE.MathUtils.clamp(time, 0, this.replay.duration)
    this.render()
    this.emitChange()
  }

  getSnapshot() {
    return {
      currentTime: this.currentTime,
      duration: this.replay.duration,
      frameIndex: findFrameIndexAtTime(this.replay, this.currentTime),
      playing: this.playing,
      speed: this.speed,
      cameraMode: this.cameraMode,
      trackedPlayerId: this.trackedPlayerId,
      ballCamEnabled: this.ballCamEnabled,
    }
  }

  dispose() {
    this.pause()
    window.removeEventListener("resize", this.boundResize)
    this.sceneState.dispose()
  }

  tick = () => {
    if (!this.playing) {
      return
    }

    const now = performance.now()
    const elapsedSeconds = (now - this.lastTickTime) / 1000
    this.lastTickTime = now
    this.currentTime += elapsedSeconds * this.speed

    if (this.currentTime >= this.replay.duration) {
      this.currentTime = this.replay.duration
      this.pause()
    }

    this.render()
    this.emitChange()
    this.animationFrameId = requestAnimationFrame(this.tick)
  }

  render() {
    const frameIndex = findFrameIndexAtTime(this.replay, this.currentTime)
    const ballFrame = this.replay.ballFrames[frameIndex]
    const ballPosition = ballFrame?.position
      ? new THREE.Vector3(
          ballFrame.position.x,
          ballFrame.position.z + BALL_RADIUS,
          ballFrame.position.y,
        )
      : null

    if (ballPosition) {
      this.sceneState.ballMesh.visible = true
      this.sceneState.ballMesh.position.copy(ballPosition)
    } else {
      this.sceneState.ballMesh.visible = false
    }

    for (const player of this.replay.players) {
      const mesh = this.sceneState.playerMeshes.get(player.id)
      if (!mesh) {
        continue
      }

      const frame = player.frames[frameIndex]
      if (!frame?.position) {
        mesh.visible = false
        continue
      }

      mesh.visible = true
      mesh.position.set(
        frame.position.x,
        frame.position.z + UP_OFFSET,
        frame.position.y,
      )

      if (frame.forward && frame.up) {
        const forward = new THREE.Vector3(
          frame.forward.x,
          frame.forward.y,
          frame.forward.z,
        ).normalize()
        const up = new THREE.Vector3(frame.up.x, frame.up.y, frame.up.z).normalize()
        const right = new THREE.Vector3().crossVectors(up, forward).normalize()
        const correctedUp = new THREE.Vector3()
          .crossVectors(forward, right)
          .normalize()
        const basis = new THREE.Matrix4().makeBasis(right, correctedUp, forward)
        mesh.quaternion.setFromRotationMatrix(basis)
      } else if (frame.velocity) {
        const heading = Math.atan2(frame.velocity.x, frame.velocity.y)
        mesh.rotation.set(0, -heading, 0)
      }
    }

    this.updateCamera(frameIndex, ballPosition)
    this.sceneState.controls.update()
    this.sceneState.renderer.render(this.sceneState.scene, this.sceneState.camera)
  }

  updateCamera(frameIndex, ballPosition) {
    const controls = this.sceneState.controls

    if (this.cameraMode === "overview" || !this.trackedPlayerId) {
      controls.enabled = true
      this.sceneState.camera.fov = 48
      this.sceneState.camera.updateProjectionMatrix()
      return
    }

    const trackedPlayer = this.replay.players.find(
      (player) => player.id === this.trackedPlayerId,
    )
    const trackedMesh = this.sceneState.playerMeshes.get(this.trackedPlayerId)
    const frame = trackedPlayer?.frames[frameIndex]

    if (!trackedPlayer || !trackedMesh || !frame?.position) {
      controls.enabled = true
      return
    }

    controls.enabled = false

    const basePosition = new THREE.Vector3(
      frame.position.x,
      frame.position.z + UP_OFFSET,
      frame.position.y,
    )
    const forward = frame.forward
      ? new THREE.Vector3(frame.forward.x, frame.forward.y, frame.forward.z)
      : DEFAULT_FORWARD.clone()
    const up = frame.up
      ? new THREE.Vector3(frame.up.x, frame.up.y, frame.up.z)
      : DEFAULT_UP.clone()
    const right = new THREE.Vector3().crossVectors(up, forward).normalize()

    const cameraSettings = trackedPlayer.cameraSettings
    const distance =
      (cameraSettings.distance ?? 270) *
      CAMERA_DISTANCE_SCALE *
      (this.cameraMode === "attached"
        ? ATTACHED_DISTANCE_MULTIPLIER
        : THIRD_PERSON_DISTANCE_MULTIPLIER)
    const height =
      (cameraSettings.height ?? 100) *
      CAMERA_HEIGHT_SCALE *
      (this.cameraMode === "attached"
        ? ATTACHED_HEIGHT_MULTIPLIER
        : THIRD_PERSON_HEIGHT_MULTIPLIER)
    const pitch = THREE.MathUtils.degToRad(cameraSettings.pitch ?? -4)
    const lookDirection = forward
      .clone()
      .applyAxisAngle(right, pitch)
      .normalize()

    this.desiredCameraPosition
      .copy(basePosition)
      .addScaledVector(forward, -distance)
      .addScaledVector(DEFAULT_UP, height)
    this.desiredCameraPosition.y = Math.max(MIN_CAMERA_HEIGHT, this.desiredCameraPosition.y)

    if (this.ballCamEnabled && ballPosition) {
      this.desiredLookTarget.copy(ballPosition).addScaledVector(DEFAULT_UP, 0.35)
    } else {
      this.desiredLookTarget
        .copy(basePosition)
        .addScaledVector(lookDirection, distance + 8)
        .addScaledVector(DEFAULT_UP, 0.8)
    }

    this.sceneState.camera.position.lerp(this.desiredCameraPosition, CAMERA_SMOOTHING)
    this.sceneState.camera.up.lerp(DEFAULT_UP, CAMERA_SMOOTHING).normalize()
    controls.target.lerp(this.desiredLookTarget, CAMERA_SMOOTHING)
    this.sceneState.camera.fov = THREE.MathUtils.lerp(
      this.sceneState.camera.fov,
      cameraSettings.fov ?? 110,
      CAMERA_SMOOTHING,
    )
    this.sceneState.camera.updateProjectionMatrix()
    this.sceneState.camera.lookAt(controls.target)
  }

  emitChange() {
    this.dispatchEvent(
      new CustomEvent("change", {
        detail: this.getSnapshot(),
      }),
    )
  }
}
