import { EventEmitter } from 'events';

export interface SocketArgs {
  host?: string;
  port: number;
  user?: string;
  password?: string;
  useBigInt?: boolean;
  enableTLS?: boolean;
  socketTimeout?: number;
  socketNoDelay?: boolean;
  includeNanosecond?: boolean;
  dateToMillisecond?: boolean;
}

export class QConnection extends EventEmitter {
  constructor(socketArgs: SocketArgs);

  readonly isConnected: boolean;

  /**
   * Connect to the kdb+ server.
   * @param callback Called with (err) on completion
   */
  connect(callback: (err: Error | null) => void): void;

  /**
   * Close the connection.
   * @param callback Called when socket closes
   */
  close(callback?: () => void): void;

  /**
   * Send a synchronous query.
   * @param param Query string or [function, ...args] array
   * @param callback Called with (err, result)
   */
  sync(param: string | any[], callback: (err: Error | null, res: any) => void): void;

  /**
   * Send an asynchronous message.
   * @param param Query string or [function, ...args] array
   * @param callback Optional error callback
   */
  asyn(param: string | any[], callback?: (err: Error | null) => void): void;
}

export declare const IPC: {
  /**
   * Deserialize a kdb+ IPC binary message.
   * @param buffer The raw IPC bytes
   * @param useBigInt Return longs as BigInt
   * @param includeNanosecond Return timestamps with nanosecond precision as strings
   * @param dateToMillisecond Return dates as millisecond numbers
   */
  deserialize(
    buffer: Uint8Array,
    useBigInt?: boolean,
    includeNanosecond?: boolean,
    dateToMillisecond?: boolean,
  ): any;

  /**
   * Serialize a JS value to kdb+ IPC binary.
   */
  serialize(obj: any): Uint8Array;
};
