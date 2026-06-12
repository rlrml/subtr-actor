import protobuf from 'protobufjs';
import type { CameraState, Vector3, Quaternion, CameraMode } from './types';

// Orbit parameters for ballOrbit mode
export interface OrbitParams {
  distance: number;
  azimuth: number;  // Horizontal angle in radians
  polar: number;    // Vertical angle in radians
}

// Camera mode enum mapping
const CAMERA_MODE_MAP: Record<string, number> = {
  free: 0,
  ballOrbit: 1,
  player: 2,
};

const CAMERA_MODE_REVERSE: Record<number, CameraMode> = {
  0: 'free',
  1: 'ballOrbit',
  2: 'player',
};

// Protocol buffer message type for CameraUpdate
let CameraUpdate: protobuf.Type | null = null;
let CameraBroadcast: protobuf.Type | null = null;

// Drawing protocol buffer types
let DrawStrokePoints: protobuf.Type | null = null;
let DrawStrokePointsBroadcast: protobuf.Type | null = null;

// Initialize protobuf types
const protoDefinition = `
syntax = "proto3";
package rlview.collab;

enum CameraMode {
  FREE = 0;
  BALL_ORBIT = 1;
  PLAYER = 2;
}

message CameraUpdate {
  float pos_x = 1;
  float pos_y = 2;
  float pos_z = 3;
  float rot_x = 4;
  float rot_y = 5;
  float rot_z = 6;
  float rot_w = 7;
  CameraMode mode = 8;
  string target_player = 9;
  float orbit_distance = 10;
  float orbit_azimuth = 11;
  float orbit_polar = 12;
}

message CameraBroadcast {
  string participant_id = 1;
  float pos_x = 2;
  float pos_y = 3;
  float pos_z = 4;
  float rot_x = 5;
  float rot_y = 6;
  float rot_z = 7;
  float rot_w = 8;
  CameraMode mode = 9;
  string target_player = 10;
  float orbit_distance = 11;
  float orbit_azimuth = 12;
  float orbit_polar = 13;
}

message Point {
  float x = 1;
  float y = 2;
  float z = 3;
}

message DrawStrokePoints {
  string stroke_id = 1;
  repeated Point points = 2;
}

message DrawStrokePointsBroadcast {
  string stroke_id = 1;
  string author_id = 2;
  repeated Point points = 3;
}
`;

// Initialize protobuf types on load
const root = protobuf.parse(protoDefinition).root;
CameraUpdate = root.lookupType('rlview.collab.CameraUpdate');
CameraBroadcast = root.lookupType('rlview.collab.CameraBroadcast');
DrawStrokePoints = root.lookupType('rlview.collab.DrawStrokePoints');
DrawStrokePointsBroadcast = root.lookupType('rlview.collab.DrawStrokePointsBroadcast');

/**
 * Encode camera state to binary format
 */
export function encodeCameraUpdate(
  position: Vector3,
  rotation: Quaternion,
  mode: CameraMode,
  targetPlayer?: string,
  orbitParams?: OrbitParams | null
): Uint8Array {
  if (!CameraUpdate) {
    throw new Error('Protocol buffer types not initialized');
  }

  // protobufjs converts snake_case to camelCase automatically
  const message = CameraUpdate.create({
    posX: position.x,
    posY: position.y,
    posZ: position.z,
    rotX: rotation.x,
    rotY: rotation.y,
    rotZ: rotation.z,
    rotW: rotation.w,
    mode: CAMERA_MODE_MAP[mode] ?? 0,
    targetPlayer: targetPlayer || '',
    // Orbit parameters for ballOrbit mode
    orbitDistance: orbitParams?.distance ?? 0,
    orbitAzimuth: orbitParams?.azimuth ?? 0,
    orbitPolar: orbitParams?.polar ?? 0,
  });

  return CameraUpdate.encode(message).finish();
}

/**
 * Decoded camera broadcast result
 */
export interface DecodedCameraBroadcast {
  participantId: string;
  camera: CameraState;
  orbitParams: OrbitParams | null;
}

/**
 * Decode camera broadcast from binary format
 */
export function decodeCameraBroadcast(
  data: Uint8Array
): DecodedCameraBroadcast | null {
  if (!CameraBroadcast) {
    throw new Error('Protocol buffer types not initialized');
  }

  try {
    const decoded = CameraBroadcast.decode(data);
    // protobufjs converts snake_case to camelCase automatically
    const message = decoded as unknown as {
      participantId: string;
      posX: number;
      posY: number;
      posZ: number;
      rotX: number;
      rotY: number;
      rotZ: number;
      rotW: number;
      mode: number;
      targetPlayer: string;
      orbitDistance: number;
      orbitAzimuth: number;
      orbitPolar: number;
    };

    // Orbit params are valid if distance is non-zero
    const hasOrbitParams = message.orbitDistance !== 0;

    return {
      participantId: message.participantId,
      camera: {
        position: {
          x: message.posX,
          y: message.posY,
          z: message.posZ,
        },
        rotation: {
          x: message.rotX,
          y: message.rotY,
          z: message.rotZ,
          w: message.rotW,
        },
        mode: CAMERA_MODE_REVERSE[message.mode] ?? 'free',
        targetPlayer: message.targetPlayer || null,
        timestamp: Date.now(),
      },
      orbitParams: hasOrbitParams ? {
        distance: message.orbitDistance,
        azimuth: message.orbitAzimuth,
        polar: message.orbitPolar,
      } : null,
    };
  } catch (error) {
    console.error('[Proto] Failed to decode camera broadcast:', error);
    return null;
  }
}

/**
 * Encode drawing stroke points to binary format (client -> server)
 */
export function encodeDrawStrokePoints(
  strokeId: string,
  points: Vector3[]
): Uint8Array {
  if (!DrawStrokePoints) {
    throw new Error('Protocol buffer types not initialized');
  }

  const message = DrawStrokePoints.create({
    strokeId,
    points: points.map((p) => ({ x: p.x, y: p.y, z: p.z })),
  });

  return DrawStrokePoints.encode(message).finish();
}

/**
 * Decoded drawing stroke points result
 */
export interface DecodedDrawStrokePoints {
  strokeId: string;
  points: Vector3[];
}

/**
 * Decode drawing stroke points from binary format (client -> server)
 */
export function decodeDrawStrokePoints(
  data: Uint8Array
): DecodedDrawStrokePoints | null {
  if (!DrawStrokePoints) {
    throw new Error('Protocol buffer types not initialized');
  }

  try {
    const decoded = DrawStrokePoints.decode(data);
    const message = decoded as unknown as {
      strokeId: string;
      points: Array<{ x: number; y: number; z: number }>;
    };

    return {
      strokeId: message.strokeId,
      points: message.points.map((p) => ({ x: p.x, y: p.y, z: p.z })),
    };
  } catch (error) {
    console.error('[Proto] Failed to decode draw stroke points:', error);
    return null;
  }
}

/**
 * Decoded drawing stroke points broadcast result (server -> clients)
 */
export interface DecodedDrawStrokePointsBroadcast {
  strokeId: string;
  authorId: string;
  points: Vector3[];
}

/**
 * Decode drawing stroke points broadcast from binary format (server -> clients)
 */
export function decodeDrawStrokePointsBroadcast(
  data: Uint8Array
): DecodedDrawStrokePointsBroadcast | null {
  if (!DrawStrokePointsBroadcast) {
    throw new Error('Protocol buffer types not initialized');
  }

  try {
    const decoded = DrawStrokePointsBroadcast.decode(data);
    const message = decoded as unknown as {
      strokeId: string;
      authorId: string;
      points: Array<{ x: number; y: number; z: number }>;
    };

    return {
      strokeId: message.strokeId,
      authorId: message.authorId,
      points: message.points.map((p) => ({ x: p.x, y: p.y, z: p.z })),
    };
  } catch (error) {
    console.error('[Proto] Failed to decode draw stroke points broadcast:', error);
    return null;
  }
}

/**
 * Check if data is binary (ArrayBuffer or Uint8Array)
 */
export function isBinaryData(data: unknown): data is ArrayBuffer | Uint8Array {
  return data instanceof ArrayBuffer || data instanceof Uint8Array;
}

/**
 * Convert ArrayBuffer to Uint8Array
 */
export function toUint8Array(data: ArrayBuffer | Uint8Array): Uint8Array {
  if (data instanceof Uint8Array) {
    return data;
  }
  return new Uint8Array(data);
}
