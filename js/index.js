/**
 * jkdb — kdb+/q interface powered by Rust/WASM.
 *
 * Connection logic in JS (net/tls sockets), codec in WASM.
 */
import { Buffer } from 'buffer';
import { EventEmitter } from 'events';
import net from 'net';
import tls from 'tls';
import { ipc_deserialize, ipc_serialize } from '../pkg/jkdb.js';

const ACK = Buffer.from('010200000a0000006500', 'hex');

export class QConnection extends EventEmitter {
  /**
   * @param {Object} socketArgs
   * @param {string}  [socketArgs.host]
   * @param {number}  socketArgs.port
   * @param {string}  [socketArgs.user]
   * @param {string}  [socketArgs.password]
   * @param {boolean} [socketArgs.useBigInt]
   * @param {boolean} [socketArgs.enableTLS]
   * @param {number}  [socketArgs.socketTimeout]
   * @param {boolean} [socketArgs.socketNoDelay]
   * @param {boolean} [socketArgs.includeNanosecond]
   * @param {boolean} [socketArgs.dateToMillisecond]
   */
  constructor(socketArgs) {
    super();
    this.socketArgs = socketArgs;
    this.host = socketArgs.host ?? 'localhost';
    this.port = socketArgs.port;
    this.user = socketArgs.user ?? '';
    this.password = socketArgs.password ?? '';
    this.useBigInt = socketArgs.useBigInt ?? false;
    /** @type {net.Socket|tls.TLSSocket|null} */
    this.socket = null;
    /** @type {function[]} */
    this.callbacks = [];
    this.socketTimeout = socketArgs.socketTimeout ?? 0;
    this.socketNoDelay = socketArgs.socketNoDelay ?? true;
    this.recvBuf = Buffer.alloc(0);
    this.enableTLS = socketArgs.enableTLS ?? false;
    this.includeNanosecond = socketArgs.includeNanosecond ?? false;
    this.dateToMillisecond = socketArgs.dateToMillisecond ?? false;
    this.isConnected = false;
  }

  setSocket(socket) {
    this.socket = socket;
    this.isConnected = true;
    this.socket.setNoDelay(this.socketNoDelay);
    this.socket.setTimeout(this.socketTimeout);
    this.socket.on('end', () => this.emit('end'));
    this.socket.on('timeout', () => this.emit('timeout'));
    this.socket.on('error', err => this.emit('error', err));
    this.socket.on('close', err => this.emit('close', err));
    this.socket.on('data', buffer => this.incomingMsgHandler(buffer));
  }

  auth(socket, callback) {
    const userPw = `${this.user}:${this.password}`;
    const n = Buffer.byteLength(userPw, 'ascii');
    const b = Buffer.alloc(n + 2);
    b.write(userPw, 0, n, 'ascii');
    b.writeUInt8(0x3, n);
    b.writeUInt8(0x0, n + 1);
    socket.write(b);
    socket.once('data', (buffer) => {
      if (buffer.length === 1) {
        if (buffer[0] >= 1) {
          socket.removeAllListeners('close');
          socket.removeAllListeners('error');
          // reset callbacks
          this.callbacks = [];
          this.setSocket(socket);
          callback(null);
          // send error to all existing callbacks
          socket.on('close', () => {
            this.callbacks.forEach(cb => cb(new Error('LOST_CONNECTION'), null));
            this.callbacks = [];
            this.isConnected = false;
          });
        } else {
          callback(new Error('UNSUPPORTED_KDB_VERSION<=2.5'));
        }
      } else {
        callback(new Error('INVALID_AUTH_RESPONSE'));
      }
    });
  }

  /**
   * @param {function(Error|null)} callback
   */
  connect(callback) {
    // if already connected, do nothing
    if (this.isConnected) {
      return callback(null);
    }
    if (this.user === '') {
      this.user = process.env.USER;
    }
    if (this.socket) {
      this.socket.end();
    }
    let socket;
    const connectListener = () => {
      // won't hit connection refused, remove error listener
      socket.removeAllListeners('error');
      socket.once('close', () => {
        socket.end();
        callback(new Error('ERR_CONNECTION_CLOSED - Wrong Credentials?'));
      });
      // connection reset by peer
      socket.once('error', err => {
        socket.end();
        callback(err);
      });
      this.auth(socket, callback);
    };

    if (this.enableTLS) {
      socket = tls.connect(this.port, this.host, { rejectUnauthorized: false }, connectListener);
    } else {
      socket = net.connect(this.port, this.host, connectListener);
    }
    // connection refused
    socket.once('error', err => callback(err));
  }

  /**
   * @param {function()} [callback]
   */
  close(callback) {
    this.socket.once('close', () => { if (callback) callback(); });
    this.socket.end();
    this.isConnected = false;
  }

  /**
   * Handle incoming TCP data.
   *
   * Accumulates chunks into recvBuf, then extracts and dispatches
   * every complete IPC message (8-byte header + payload).
   *
   * @param {Buffer} chunk
   */
  incomingMsgHandler(chunk) {
    this.recvBuf = this.recvBuf.length === 0
      ? chunk
      : Buffer.concat([this.recvBuf, chunk]);

    // Process all complete messages in the buffer
    while (this.recvBuf.length >= 8) {
      const msgLen = this.recvBuf.readUInt32LE(4);
      if (this.recvBuf.length < msgLen) break; // wait for more data

      const msg = this.recvBuf.subarray(0, msgLen);
      this.recvBuf = this.recvBuf.subarray(msgLen);

      let obj, err;
      try {
        obj = ipc_deserialize(
          new Uint8Array(msg),
          this.useBigInt,
          this.includeNanosecond,
          this.dateToMillisecond,
        );
        err = null;
      } catch (e) {
        obj = null;
        err = e;
      }

      const msgType = msg[1];
      if (msgType === 2) {
        // response msg
        this.callbacks.shift()(err, obj);
      } else if (msgType === 0) {
        // async msg, no ack needed
        if (!err && Array.isArray(obj) && obj[0] === 'upd') {
          this.emit('upd', obj);
        }
      } else {
        // sync msg from server — ack it
        this.socket.write(ACK);
      }
    }
  }

  /**
   * @param {string|Array} param
   * @param {function(Error, any)} callback
   */
  sync(param, callback) {
    if (typeof callback !== 'function') {
      throw new Error('Expecting a callback function as last param');
    }

    // null or empty list of param
    if (!param || (Array.isArray(param) && param.length === 0)) {
      this.callbacks.push(callback);
    } else {
      const buffer = Buffer.from(ipc_serialize(param));
      // sync(1) msg
      buffer.writeUInt8(0x1, 1);
      this.socket.write(buffer, () => this.callbacks.push(callback));
    }
  }

  /**
   * @param {string|Array} param
   * @param {function(Error)} [callback]
   */
  asyn(param, callback) {
    const buffer = Buffer.from(ipc_serialize(param));
    // async(0) msg
    buffer.writeUInt8(0x1, 0);
    if (callback) {
      this.socket.write(buffer, callback);
    } else {
      this.socket.write(buffer);
    }
  }
}

// Re-export IPC functions for direct use
export const IPC = {
  deserialize: ipc_deserialize,
  serialize: ipc_serialize,
};
