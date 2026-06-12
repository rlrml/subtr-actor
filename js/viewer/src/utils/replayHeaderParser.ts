/**
 * Lightweight client-side parser for Rocket League replay headers
 * Extracts just the ReplayName property without full parsing
 *
 * Rocket League replay format (Unreal Engine serialization):
 * - Header size (4 bytes LE)
 * - Header CRC (4 bytes)
 * - Major version (4 bytes)
 * - Minor version (4 bytes)
 * - Net version (4 bytes, if major >= 868)
 * - Game type (length-prefixed string)
 * - Properties (key-value pairs until "None")
 */

interface ReplayHeaderInfo {
  replayName: string | null;
  mapName: string | null;
  matchType: string | null;
}

/**
 * Read a length-prefixed string from the buffer
 * Format: 4-byte length (LE), then UTF-8 or Windows-1252 string
 */
function readString(view: DataView, offset: number): [string, number] {
  const length = view.getInt32(offset, true);
  offset += 4;

  if (length === 0) {
    return ['', offset];
  }

  // Negative length indicates UTF-16
  if (length < 0) {
    const actualLength = -length;
    const decoder = new TextDecoder('utf-16le');
    const bytes = new Uint8Array(view.buffer, view.byteOffset + offset, actualLength * 2);
    const str = decoder.decode(bytes).replace(/\0+$/, ''); // Remove null terminator
    return [str, offset + actualLength * 2];
  }

  // Positive length is UTF-8/ASCII
  const bytes = new Uint8Array(view.buffer, view.byteOffset + offset, length);
  // Try to decode as UTF-8, fall back to latin1 for special chars
  let str: string;
  try {
    str = new TextDecoder('utf-8').decode(bytes);
  } catch {
    str = new TextDecoder('windows-1252').decode(bytes);
  }
  // Remove null terminator
  str = str.replace(/\0+$/, '');
  return [str, offset + length];
}

/**
 * Read a property value based on its type
 */
function readPropertyValue(
  view: DataView,
  offset: number,
  propType: string
): [unknown, number] {
  switch (propType) {
    case 'IntProperty': {
      // Size (8 bytes) + value (4 bytes)
      offset += 8; // Skip size
      const value = view.getInt32(offset, true);
      return [value, offset + 4];
    }
    case 'StrProperty':
    case 'NameProperty': {
      // Size (8 bytes) + string
      offset += 8; // Skip size
      const [str, newOffset] = readString(view, offset);
      return [str, newOffset];
    }
    case 'FloatProperty': {
      offset += 8; // Skip size
      const value = view.getFloat32(offset, true);
      return [value, offset + 4];
    }
    case 'BoolProperty': {
      offset += 8; // Skip size
      const value = view.getUint8(offset) !== 0;
      return [value, offset + 1];
    }
    case 'ByteProperty': {
      offset += 8; // Skip size
      // Read enum type name
      const [enumType, offset2] = readString(view, offset);
      if (enumType === 'None') {
        const value = view.getUint8(offset2);
        return [value, offset2 + 1];
      }
      // Read enum value name
      const [enumValue, offset3] = readString(view, offset2);
      return [enumValue, offset3];
    }
    case 'QWordProperty': {
      offset += 8; // Skip size
      // Read as BigInt64
      const low = view.getUint32(offset, true);
      const high = view.getUint32(offset + 4, true);
      return [BigInt(high) * BigInt(0x100000000) + BigInt(low), offset + 8];
    }
    case 'ArrayProperty': {
      // For arrays, we just skip them as we don't need PlayerStats etc.
      const size = Number(view.getBigUint64(offset, true));
      offset += 8;
      // Skip the array content
      return [null, offset + size];
    }
    default: {
      // Unknown type, try to skip using size
      const size = Number(view.getBigUint64(offset, true));
      offset += 8;
      return [null, offset + size];
    }
  }
}

/**
 * Parse replay header to extract basic info
 * Only reads until we find the properties we need
 */
export async function parseReplayHeader(file: File): Promise<ReplayHeaderInfo> {
  const result: ReplayHeaderInfo = {
    replayName: null,
    mapName: null,
    matchType: null,
  };

  try {
    // Read first 64KB - should be enough for header properties
    const headerSize = Math.min(file.size, 65536);
    const buffer = await file.slice(0, headerSize).arrayBuffer();
    const view = new DataView(buffer);

    let offset = 0;

    // Skip header size and CRC
    offset += 8;

    // Read major and minor version
    const majorVersion = view.getUint32(offset, true);
    offset += 4;
    offset += 4; // minor version

    // Net version (only present in newer replays)
    if (majorVersion >= 868) {
      offset += 4;
    }

    // Skip game type string
    const [, offset2] = readString(view, offset);
    offset = offset2;

    // Now read properties until we find what we need or hit "None"
    const maxIterations = 100; // Safety limit
    for (let i = 0; i < maxIterations; i++) {
      // Read property name
      const [propName, offset3] = readString(view, offset);
      offset = offset3;

      if (propName === 'None' || propName === '') {
        break;
      }

      // Read property type
      const [propType, offset4] = readString(view, offset);
      offset = offset4;

      // Read property value
      const [value, offset5] = readPropertyValue(view, offset, propType);
      offset = offset5;

      // Check if this is a property we care about
      if (propName === 'ReplayName' && typeof value === 'string') {
        result.replayName = value;
      } else if (propName === 'MapName' && typeof value === 'string') {
        result.mapName = value;
      } else if (propName === 'MatchType' && typeof value === 'string') {
        result.matchType = value;
      }

      // Early exit if we found everything we need
      if (result.replayName && result.mapName && result.matchType) {
        break;
      }
    }
  } catch (error) {
    console.warn('[ReplayHeaderParser] Failed to parse replay header:', error);
  }

  return result;
}
