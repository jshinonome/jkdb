import { Buffer } from 'buffer';
import { EventEmitter } from 'events';
import net from 'net';
import tls from 'tls';
import IPC from './ipc';

export class QConnection extends EventEmitter {
  /**
 * @constructs socketArgs
 * @param  {Object}    socketArgs
 * @param  {string}    [socketArgs.host]
 * @param  {number}    socketArgs.port
 * @param  {string}    [socketArgs.user]
 * @param  {string}    [socketArgs.password]
 * @param  {boolean}   [socketArgs.useBigInt]
 * @param  {boolean}   [socketArgs.enableTLS]
 * @param  {boolean}   [socketArgs.socketTimeout]
 * @param  {boolean}   [socketArgs.includeNanosecond]
 * @param  {boolean}   [socketArgs.dateToMillisecond]
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
    this.msgBuffer = Buffer.alloc(0);
    this.msgOffset = 0;
    this.enableTLS = socketArgs.enableTLS ?? false;
    this.includeNanosecond = socketArgs.includeNanosecond ?? false;
    this.dateToMillisecond = socketArgs.dateToMillisecond ?? false;
  }

  setSocket(socket) {
    this.socket = socket;
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
   *
   * @callback errorHandler
   * @param {Error} err
   */

  /**
   *
   * @param {errorHandler} callback
   */
  connect(callback) {
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
   *
   * @param {function()} [callback]
   */
  close(callback) {
    this.socket.once('close', () => { if (callback) callback(); });
    this.socket.end();
  }

  /**
   *
   * @param {Buffer} buffer
   */
  incomingMsgHandler(buffer) {
    if (this.msgBuffer.length > 8) {
      buffer.copy(this.msgBuffer, this.msgOffset);
      this.msgOffset += buffer.length;
    } else if (this.msgBuffer.length === 0 && buffer.length >= 8) {
      const length = buffer.readUInt32LE(4);
      if (length > buffer.length) {
        this.msgBuffer = Buffer.alloc(length);
        buffer.copy(this.msgBuffer);
        this.msgOffset = buffer.length;
        return;
      } else {
        this.msgBuffer = buffer.subarray(0, length);
        this.msgOffset = buffer.length;
      }
    } else if (this.msgBuffer.length + buffer.length >= 8) {
      const buf = Buffer.alloc(8);
      this.msgBuffer.copy(buf);
      buffer.copy(buf, this.msgBuffer.length);
      const length = buf.readUInt32LE(4);
      this.msgBuffer = Buffer.alloc(length);
      buf.copy(this.msgBuffer);
      buffer.copy(this.msgBuffer, this.msgBuffer.length + buffer.length - 8);
      this.msgOffset = this.msgBuffer.length + buffer.length;
    } else {
      // overall length < 8
      const buf = Buffer.alloc(this.msgBuffer.length + buffer.length);
      this.msgBuffer.copy(buf);
      buffer.copy(buf, this.msgBuffer.length);
      this.msgBuffer = buf;
      this.msgOffset = 0;
    }

    while (this.msgOffset > 0 && this.msgOffset >= this.msgBuffer.length) {
      let obj, err;
      try {
        obj = IPC.deserialize(this.msgBuffer, this.useBigInt, this.includeNanosecond, this.dateToMillisecond);
        err = null;
      } catch (e) {
        obj = null;
        err = e;
      }
      // response(2) msg
      if (this.msgBuffer.readUInt8(1) === 2) {
        this.callbacks.shift()(err, obj);
      } else {
        if (!err && Array.isArray(obj) && obj[0] === 'upd') {
          this.emit('upd', obj);
        } else {
          this.callbacks.shift()(err, obj);
        }
      }
      if (this.msgOffset > this.msgBuffer.length) {
        const subBuf = buffer.subarray(buffer.length + this.msgBuffer.length - this.msgOffset);
        if (subBuf.length >= 8) {
          const length = subBuf.readUInt32LE(4);
          if (length > subBuf.length) {
            const buf = Buffer.alloc(length);
            subBuf.copy(buf);
            this.msgBuffer = buf;
          } else {
            this.msgBuffer = subBuf;
          }
          this.msgOffset = subBuf.length;
        } else {
          this.msgBuffer = subBuf;
          this.msgOffset = 0;
        }
      } else {
        this.msgBuffer = Buffer.alloc(0);
        this.msgOffset = 0;
      }
    }
  }

  /**
   *
   * @callback queryHandler
   * @param {Error} err
   * @param {any} res
   */

  /**
   *
   * @param {string|Array} param
   * @param {queryHandler} callback
   */
  sync(param, callback) {
    if (typeof callback !== 'function') {
      throw new Error('Expecting a callback function as last param');
    }

    // null or empty list of param
    if (!param || (Array.isArray(param) && param.length === 0)) {
      this.callbacks.push(callback);
    } else {
      const buffer = IPC.serialize(param);
      // sync(1) msg
      buffer.writeUInt8(0x1, 1);
      this.socket.write(buffer, () => this.callbacks.push(callback));
    }
  }

  /**
   *
   * @param {string|Array} param
   * @param {errorHandler} [callback]
   */
  asyn(param, callback) {
    const buffer = IPC.serialize(param);
    // async(0) msg
    buffer.writeUInt8(0x1, 0);
    if (callback) {
      this.socket.write(buffer, callback);
    } else {
      this.socket.write(buffer);
    }
  }
}
