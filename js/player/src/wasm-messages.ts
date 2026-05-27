import type { ReplayLoadProgress } from "./types";

export type ReplayValidation = {
  valid: boolean;
  message?: string;
  error?: string;
};

export interface ReplayLoadRequest {
  type: "load-replay";
  bytes: ArrayBuffer;
  reportEveryNFrames: number;
}

export interface ReplayProgressMessage {
  type: "progress";
  progress: ReplayLoadProgress;
}

export interface ReplayDoneMessage {
  type: "done";
  rawBuffer: ArrayBuffer;
  replayBuffer: ArrayBuffer;
}

export interface ReplayErrorMessage {
  type: "error";
  error: string;
}

export type ReplayWorkerMessage = ReplayProgressMessage | ReplayDoneMessage | ReplayErrorMessage;
