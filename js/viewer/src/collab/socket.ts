import { io, Socket } from 'socket.io-client';
import { COLLAB_CONFIG } from './types';

let socket: Socket | null = null;

/**
 * Get or create Socket.IO connection to collab namespace
 */
export function getCollabSocket(): Socket {
  return getSocket();
}

/**
 * Alias for getCollabSocket for convenience
 */
export function getSocket(): Socket {
  // Return existing socket if it exists (even if disconnected)
  if (socket) {
    return socket;
  }

  const wsUrl = import.meta.env.VITE_WS_URL || window.location.origin;
  console.log('[Collab] Creating socket to:', `${wsUrl}${COLLAB_CONFIG.NAMESPACE}`);

  socket = io(`${wsUrl}${COLLAB_CONFIG.NAMESPACE}`, {
    autoConnect: false,
    reconnection: true,
    reconnectionAttempts: 5,
    reconnectionDelay: 1000,
    reconnectionDelayMax: 5000,
    timeout: 10000,
    transports: ['websocket', 'polling'],
    // Send cookies with request for httpOnly token authentication (021-auth-infra-improvements)
    withCredentials: true,
  });

  // Connection event handlers
  socket.on('connect', () => {
    console.log('[Collab] Connected to server');
  });

  socket.on('disconnect', (reason) => {
    console.log('[Collab] Disconnected:', reason);
  });

  socket.on('connect_error', (error) => {
    console.error('[Collab] Connection error:', error.message);
  });

  socket.on('reconnect', (attemptNumber) => {
    console.log('[Collab] Reconnected after', attemptNumber, 'attempts');
  });

  socket.on('reconnect_attempt', (attemptNumber) => {
    console.log('[Collab] Reconnection attempt', attemptNumber);
  });

  socket.on('reconnect_failed', () => {
    console.error('[Collab] Reconnection failed');
  });

  return socket;
}

/**
 * Connect to the collab server
 */
export function connectCollab(): void {
  const sock = getCollabSocket();
  if (!sock.connected) {
    sock.connect();
  }
}

/**
 * Disconnect from the collab server
 */
export function disconnectCollab(): void {
  if (socket) {
    socket.disconnect();
    socket = null;
  }
}

/**
 * Check if socket is connected
 */
export function isCollabConnected(): boolean {
  return socket?.connected ?? false;
}

/**
 * Get socket ID
 */
export function getSocketId(): string | null {
  return socket?.id ?? null;
}

/**
 * Emit an event and return a promise for the acknowledgement
 */
export function emitWithAck<T>(
  event: string,
  data?: unknown
): Promise<T> {
  return new Promise((resolve, reject) => {
    const sock = getCollabSocket();
    console.log(`[Collab] *** emitWithAck: event=${event}, connected=${sock.connected}, data=`, data);
    if (!sock.connected) {
      console.error(`[Collab] *** emitWithAck REJECTED: socket not connected`);
      reject(new Error('Not connected to collab server'));
      return;
    }

    sock.emit(event, data, (response: T) => {
      console.log(`[Collab] *** emitWithAck response for ${event}:`, response);
      resolve(response);
    });
  });
}

/**
 * Emit binary data (for camera updates)
 */
export function emitBinary(event: string, data: ArrayBuffer | Uint8Array): void {
  const sock = getCollabSocket();
  if (sock.connected) {
    sock.emit(event, data);
  }
}
