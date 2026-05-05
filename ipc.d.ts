/**
 * Deserialize a kdb+ IPC binary message.
 */
export function ipc_deserialize(
  buffer: Uint8Array,
  useBigInt?: boolean,
  includeNanosecond?: boolean,
  dateToMillisecond?: boolean,
): any;

/**
 * Serialize a JS value to kdb+ IPC binary.
 */
export function ipc_serialize(obj: any): Uint8Array;
