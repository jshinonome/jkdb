const SHORT_NULL = -32768;
const SHORT_POSITIVE_INFINITY = 32767;
const SHORT_NEGATIVE_INFINITY = -32767;
const INT_NULL = -2147483648;
const INT_POSITIVE_INFINITY = 2147483647;
const INT_NEGATIVE_INFINITY = -2147483647;
const LONG_NULL = -9223372036854775808n;
const LONG_POSITIVE_INFINITY = 9223372036854775807n;
const LONG_NEGATIVE_INFINITY = -9223372036854775807n;
const MS_DIFF = 946684800000;
const MS_PER_DAY = 86400000;
const K_TYPE_CHAR = ' bg xhijefcspmdznuvt';

const SIZE_BY_K_TYPE = {
  1: 1,
  2: 16,
  4: 1,
  5: 2,
  6: 4,
  7: 8,
  8: 4,
  9: 8,
  10: 1,
  12: 8,
  13: 4,
  14: 4,
  15: 8,
  16: 8,
  17: 4,
  18: 4,
  19: 4,
  101: 1,
};

const K101 = {
  0: null,
  1: '+:',
  2: '-:',
  3: '*:',
  4: '%:',
  5: '&:',
  6: '|:',
  7: '^:',
  8: '=:',
  9: '<:',
  10: '>:',
  11: '$:',
  12: ',:',
  13: '#:',
  14: '_:',
  15: '~:',
  16: '!:',
  17: '?:',
  18: '@:',
  19: '.:',
  20: '0::',
  21: '1::',
  22: '2::',
  23: 'avg',
  24: 'last',
  25: 'sum',
  26: 'prd',
  27: 'min',
  28: 'max',
  29: 'exit',
  30: 'getenv',
  31: 'abs',
  32: 'sqrt',
  33: 'log',
  34: 'exp',
  35: 'sin',
  36: 'asin',
  37: 'cos',
  38: 'acos',
  39: 'tan',
  40: 'atan',
  41: 'enlist',
  42: 'var',
  43: 'dev',
  44: 'hopen',
  // projection null
  255: '::',
};

const K102 = {
  0: ':',
  1: '+',
  2: '-',
  3: '*',
  4: '%',
  5: '&',
  6: '|',
  7: '^',
  8: '=',
  9: '<',
  10: '>',
  11: '$',
  12: ',',
  13: '#',
  14: '_',
  15: '~',
  16: '!',
  17: '?',
  18: '@',
  19: '.',
  20: '0:',
  21: '1:',
  22: '2:',
  23: 'in',
  24: 'within',
  25: 'like',
  26: 'bin',
  27: 'ss',
  28: 'insert',
  29: 'wsum',
  30: 'wavg',
  31: 'div',
  32: 'xexp',
  33: 'setenv',
  34: 'binr',
  35: 'cov',
  36: 'cor',
};

const K103 = {
  0: '\'',
  1: '/',
  2: '\\',
  3: '\':',
  4: '/:',
  5: '\\:',
};

/**
 *
 * @param {Buffer} cMsg
 * @returns {Buffer}
 */
function decompress(cMsg) {
  const oLen = cMsg.readInt32LE(8);
  const msg = Buffer.alloc(oLen);
  cMsg.copy(msg, 0, 0, 4);
  msg[2] = 0;
  cMsg.copy(msg, 4, 8, 12);
  let cPos = 12, oPos = 8, xPos = oPos, n = 0, s = 0, r = 0, i = 0, j = 0;
  const x = new Int32Array(256);
  while (oPos < oLen) {
    if (i === 0) {
      n = cMsg[cPos++];
      i = 1;
    }
    r = 0;
    if (n & i) {
      s = x[cMsg[cPos++]];
      r = cMsg[cPos++];
      for (j = 0; j < r + 2; j++) {
        msg[oPos + j] = msg[s + j];
      }
      oPos += 2;
    } else {
      msg[oPos++] = cMsg[cPos++];
    }

    while (xPos < (oPos - 1)) {
      x[msg[xPos] ^ msg[xPos + 1]] = xPos++;
    }
    if (n & i) {
      xPos = (oPos += r);
    }
    i *= 2;
    if (i === 256) {
      i = 0;
    }
  }
  return msg;
}

/**
 *
 * @param {BigInt} ns
 * @returns {string|null}
 */
function bigintToTimespan(ns) {
  if (!Number.isNaN(ns)) {
    const sign = ns < 0n ? '-' : '';
    if (ns < 0n) {
      ns = -1n * ns;
    }
    const second = ns / 1000000000n;
    return `${sign}${ns / 86400000000000n}D${String(second / 3600n % 24n).padStart(2, '0')}:${String(second / 60n % 60n).padStart(2, '0')}:${String(second % 60n).padStart(2, '0')}.${String(ns % 1000000000n).padStart(9, '0')}`;
  } else {
    return null;
  }
}

function intToTemporal(unit, kType) {
  if (!Number.isFinite(unit)) {
    return null;
  }

  let yyyy, MM, hh, mm, ss, SSS;
  const sign = unit < 0 ? '-' : '';
  const absUnit = Math.abs(unit);
  switch (kType) {
    // month
    case 243:
    case 13:
      yyyy = 2000 + (unit / 12) >>> 0;
      MM = unit % 12;
      return `${yyyy}.${String(MM + (MM < 0 ? 13 : 1)).padStart(2, '0')}m`;
    // minute
    case 239:
    case 17:
      hh = (absUnit / 60) >>> 0;
      mm = absUnit % 60;
      return sign + [hh, mm].map(val => String(val).padStart(2, '0')).join(':');
    // second
    case 238:
    case 18:
      hh = (absUnit / 3600) >>> 0;
      mm = (absUnit / 60) % 60 >>> 0;
      ss = absUnit % 60;
      return sign + [hh, mm, ss].map(val => String(val).padStart(2, '0')).join(':');
    // ms
    case 237:
    case 19:
      hh = (absUnit / 3600000) >>> 0;
      mm = (absUnit / 60000) % 60 >>> 0;
      ss = absUnit / 1000 % 60 >>> 0;
      SSS = absUnit % 1000;
      return sign + [hh, mm, ss].map(val => String(val).padStart(2, '0')).join(':') + '.' + String(SSS).padStart(3, '0');
    default:
      throw new Error('FAILED_TO_CAST_TO_TEMPORAL_STRING - ' + kType);
  }
}

function convertI16(i16) {
  switch (i16) {
    case SHORT_NULL:
      return NaN;
    case SHORT_POSITIVE_INFINITY:
      return Infinity;
    case SHORT_NEGATIVE_INFINITY:
      return -Infinity;
    default:
      return i16;
  }
}

function convertI32(i32) {
  switch (i32) {
    case INT_NULL:
      return NaN;
    case INT_POSITIVE_INFINITY:
      return Infinity;
    case INT_NEGATIVE_INFINITY:
      return -Infinity;
    default:
      return i32;
  }
}

function convertBigI64(bigI64) {
  switch (bigI64) {
    case LONG_NULL:
      return NaN;
    case LONG_POSITIVE_INFINITY:
      return Infinity;
    case LONG_NEGATIVE_INFINITY:
      return -Infinity;
    default:
      return bigI64;
  }
}

/**
 *
 * @param {Buffer} buffer
 * @param {boolean} useBigInt
 * @param {boolean} includeNanosecond
 * @param {boolean} dateToMillisecond
 * @returns {any}
 */
function deserialize(buffer, useBigInt = false, includeNanosecond = false, dateToMillisecond = false) {
  let offset = 8;
  const readAtomByKType = (kType, useBigInt = false) => {
    switch (kType) {
      case 255:
        // boolean
        return buffer[offset++] === 1;
      case 254: {
        const guid = buffer.subarray(offset, offset + 16).toString('hex');
        offset += 16;
        return guid;
      }
      case 252:
        // byte
        return buffer[offset++];
      case 251: {
        // short
        const i16 = buffer.readInt16LE(offset);
        offset += 2;
        return convertI16(i16);
      }
      // int 32
      case 250: {
        const i32 = buffer.readInt32LE(offset);
        offset += 4;
        return convertI32(i32);
      }
      case 249: {
        const bigI64 = convertBigI64(buffer.readBigInt64LE(offset));
        offset += 8;
        return useBigInt ? bigI64 : Number(bigI64);
      }
      case 248: {
        const real = buffer.readFloatLE(offset);
        offset += 4;
        return real;
      }
      case 247: {
        const float = buffer.readDoubleLE(offset);
        offset += 8;
        return float;
      }
      case 246:
        return String.fromCharCode(buffer[offset++]);
      // symbol
      case 245: {
        const end = buffer.indexOf(0, offset);
        const sym = buffer.subarray(offset, end).toString();
        offset = end + 1;
        return sym;
      }
      // timestamp
      case 244: {
        const ns = readAtomByKType(249, true);
        if (typeof ns === 'bigint') {
          const date = new Date(Number(ns / 1000000n) + MS_DIFF);
          return includeNanosecond ? date.toISOString().slice(0, -1) + String(ns % 1000000n).padStart(6, '0') : date;
        } else {
          return includeNanosecond ? '' : null;
        }
      }
      case 243: {
        const month = readAtomByKType(250);
        return intToTemporal(month, 243);
      }
      // date
      case 242: {
        const ms = MS_DIFF + readAtomByKType(250) * MS_PER_DAY;
        return dateToMillisecond ? ms : Number.isFinite(ms) ? new Date(ms) : null;
      }
      // datetime
      case 241: {
        const ms = MS_DIFF + readAtomByKType(247) * MS_PER_DAY;
        return dateToMillisecond ? ms : Number.isFinite(ms) ? new Date(ms) : null;
      }
      case 240: {
        const ns = readAtomByKType(249, true);
        return typeof ns === 'bigint' ? bigintToTimespan(ns) : null;
      }
      // minute
      case 239: {
        const minute = readAtomByKType(250);
        return intToTemporal(minute, 239);
      }
      // second
      case 238: {
        const second = readAtomByKType(250);
        return intToTemporal(second, 238);
      }
      // ms
      case 237: {
        const ms = readAtomByKType(250);
        return intToTemporal(ms, 237);
      }
    }
  };

  const readArray = kType => {
    // skip attribute
    offset++;
    const n = readAtomByKType(250);

    if (kType === 10) {
      // string, char list
      const str = buffer.subarray(offset, offset + n).toString();
      offset += n;
      return str;
    }

    const array = new Array(n);

    // array with dynamic length atom
    if (kType === 0) {
      for (i = 0; i < n; i++) {
        array[i] = read();
      }
      return array;
    } else if (kType === 11) {
      for (i = 0; i < n; i++) {
        array[i] = readAtomByKType(245);
      }
      return array;
    } else if (kType === 2) {
      for (i = 0; i < n; i++) {
        array[i] = buffer.subarray(offset + i * 16, offset + (i + 1) * 16).toString('hex');
      }
      offset += 16 * n;
      return array;
    }

    // read fixed length array
    // DataView > Buffer.read > new TypedArray
    const dv = new DataView(buffer.buffer, offset + buffer.offset, SIZE_BY_K_TYPE[kType] * n);
    const size = SIZE_BY_K_TYPE[kType];
    offset += n * size;
    let i = 0;
    switch (kType) {
      case 1:
        for (i = 0; i < n; i++) {
          array[i] = dv.getUint8(i) === 1;
        }
        return array;
      case 4:
        for (i = 0; i < n; i++) {
          array[i] = dv.getUint8(i);
        }
        return array;
      case 5:
        for (i = 0; i < n; i++) {
          const i16 = dv.getInt16(i * size, true);
          array[i] = convertI16(i16);
        }
        return array;
      case 6:
        for (i = 0; i < n; i++) {
          const i32 = dv.getInt32(i * size, true);
          array[i] = convertI32(i32);
        }
        return array;
      case 7:
        if (useBigInt) {
          for (i = 0; i < n; i++) {
            const bigI64 = dv.getBigInt64(i * size, true);
            array[i] = convertBigI64(bigI64);
          }
        } else {
          for (i = 0; i < n; i++) {
            const bigI64 = dv.getBigInt64(i * size, true);
            array[i] = Number(convertBigI64(bigI64));
          }
        }
        return array;
      case 8:
        for (i = 0; i < n; i++) {
          array[i] = dv.getFloat32(i * size, true);
        }
        return array;
      case 9:
        for (i = 0; i < n; i++) {
          array[i] = dv.getFloat64(i * size, true);
        }
        return array;
      // timestamp
      case 12:
        for (i = 0; i < n; i++) {
          const ns = dv.getBigInt64(i * size, true);
          if (ns === LONG_NULL || ns === LONG_POSITIVE_INFINITY || ns === LONG_NEGATIVE_INFINITY) {
            array[i] = includeNanosecond ? '' : null;
          } else {
            const date = new Date(Number(ns / 1000000n) + MS_DIFF);
            array[i] = includeNanosecond ? date.toISOString().slice(0, -1) + String(ns % 1000000n).padStart(6, '0') : date;
          }
        }
        return array;
      case 13:
      case 17:
      case 18:
      case 19:
        for (i = 0; i < n; i++) {
          const i32 = dv.getInt32(i * size, true);
          const unit = convertI32(i32);
          array[i] = Number.isFinite(unit) ? intToTemporal(unit, kType) : null;
        }
        return array;
      // date
      case 14:
        for (i = 0; i < n; i++) {
          const i32 = dv.getInt32(i * size, true);
          const ms = MS_DIFF + convertI32(i32) * MS_PER_DAY;
          array[i] = dateToMillisecond ? ms : Number.isFinite(ms) ? new Date(ms) : null;
        }
        return array;
      // datetime
      case 15:
        for (i = 0; i < n; i++) {
          const ms = MS_DIFF + dv.getFloat64(i * size, true) * MS_PER_DAY;
          array[i] = dateToMillisecond ? ms : Number.isFinite(ms) ? new Date(ms) : null;
        }
        return array;
      case 16:
        for (i = 0; i < n; i++) {
          const ns = dv.getBigInt64(i * size, true);
          if (ns === LONG_NULL || ns === LONG_POSITIVE_INFINITY || ns === LONG_NEGATIVE_INFINITY) {
            array[i] = null;
          } else {
            array[i] = bigintToTimespan(ns);
          }
        }
        return array;
      default:
        throw new Error('UNSUPPORTED_K_LIST - ' + kType);
    }
  };

  const read = (flipTable = false) => {
    // kType
    const kType = buffer[offset++];
    if (kType === 128) {
      throw new Error(readAtomByKType(245));
    }
    if (237 <= kType && kType <= 255) {
      return readAtomByKType(kType, useBigInt);
    }
    if (0 <= kType && kType <= 19) {
      const array = readArray(kType);
      if (Array.isArray(array)) {
        array[Symbol.for('kType')] = K_TYPE_CHAR[kType];
      }
      return array;
    }
    // dict, keyed table
    if (kType === 99) {
      const isKeyTable = buffer[offset] === 98;
      const k = read();
      // key is not a table, treat value table as a list of dictionaries
      const v = k[Symbol.for('meta')] ? read() : read(flipTable = true);
      if (isKeyTable) {
        for (const [key, value] of Object.entries(v)) {
          k[key] = value;
        }
        const meta = k[Symbol.for('meta')];
        k[Symbol.for('keys')] = meta.c.slice();
        const vMeta = v[Symbol.for('meta')];
        meta.c.push(...vMeta.c);
        meta.t.push(...vMeta.t);
        k[Symbol.for('meta')] = meta;
        return k;
      } else {
        const dict = {};
        k.forEach((k, i) => dict[k] = v[i]);
        return dict;
      }
    }
    // table
    if (kType === 98) {
      // skip to read columns(symbol) - attribute, dict k type, sym k type
      offset += 3;
      const column = readArray(11);
      // skip mixed list, attribute, length
      offset += 6;
      const meta = { c: column, t: [] };
      const table = {};
      column.forEach(c => {
        const kType = buffer[offset++];
        table[c] = readArray(kType);
        meta.t.push(K_TYPE_CHAR[kType]);
      });
      // append meta
      table[Symbol.for('meta')] = meta;
      if (flipTable) {
        const rows = new Array(table[meta.c[0]].length);
        table[meta.c[0]].forEach(
          (_v, i) => {
            const row = {};
            meta.c.forEach(
              (col) => row[col] = table[col][i]
            );
            rows[i] = row;
          });
        return rows;
      } else {
        return table;
      }
    }

    if (kType === 100) {
      if (buffer[offset++] > 0) {
        offset++;
      }
      offset++;
      return readArray(10);
    }

    if (kType === 101) {
      // unary primitive
      return K101[buffer[offset++]];
    }

    if (kType === 102) {
      // operator
      return K102[buffer[offset++]];
    }

    if (kType === 103) {
      // iterator
      return K103[buffer[offset++]];
    }

    if (kType === 104) {
      // no attribute to skip, move back offset by 1
      offset--;
      return readArray(0);
    }

    throw new Error('UNSUPPORTED_K_TYPE[read] - ' + kType);
  };

  if (buffer[2] === 1) {
    buffer = decompress(buffer);
  }
  return read();
}

function getKType(x) {
  if (x === null || x === undefined) {
    return 101;
  }
  if (x instanceof Date) {
    return 244;
  }
  if (Array.isArray(x)) {
    const kTypeChar = x[Symbol.for('kType')];
    const kType = kTypeChar ? K_TYPE_CHAR.indexOf(kTypeChar) : 0;
    return kType > 0 ? kType : 0;
  }
  switch (typeof x) {
    // float
    case 'number':
      return 247;
    case 'bigint':
      return 249;
    case 'boolean':
      return 255;
    case 'string':
      return 10;
    case 'object': {
      const meta = x[Symbol.for('meta')];
      if (meta) {
        return 98;
      } else {
        return 99;
      }
    }
  }
}

function calcMsgLength(obj, kType = null) {
  kType = kType ?? getKType(obj);
  if (kType !== 10 && kType < 20 && !Array.isArray(obj)) {
    throw new Error('NOT_AN_ARRAY[calcMsgLength]');
  }
  switch (kType) {
    case 101:
      return 1 + SIZE_BY_K_TYPE[kType];
    case 244:
    case 246:
    case 247:
    case 249:
    case 255:
      return 1 + SIZE_BY_K_TYPE[256 - kType];
    case 98: {
      let length = 3;
      const meta = obj[Symbol.for('meta')];
      const column = meta.c;
      length += calcMsgLength(column, 11);
      const size = obj[column[0]].length;
      // values: general list
      length += 6;
      column.forEach((c, i) => {
        if (size !== obj[c].length) {
          throw new Error('NOT_SAME_SIZE_COLUMN - ' + c);
        }
        let t = K_TYPE_CHAR.indexOf(meta.t[i]);
        t = t > 0 ? t : 0;
        length += calcMsgLength(obj[c], t);
      });
      return length;
    }
    case 99: {
      const k = Object.keys(obj);
      const v = Object.values(obj);
      return 1 + calcMsgLength(k, 11) + calcMsgLength(v);
    }
    case 0: {
      let length = 6;
      obj.forEach(item => length += calcMsgLength(item));
      return length;
    }
    // symbol
    case 11: {
      let length = 6;
      obj.forEach(item => length += item.length + 1);
      return length;
    }
    case 10:
      return 6 + Buffer.byteLength(obj, 'utf8');
    case 1:
    case 2:
    case 6:
    case 7:
    case 9:
    case 12:
    case 14:
    case 15:
      return 6 + SIZE_BY_K_TYPE[kType] * obj.length;
    default:
      throw new Error('UNSUPPORTED_K_TYPE[calcMsgLength] - ' + kType);
  }
}

/**
 *
 * @param {any} obj
 * @returns {Buffer}
 */
function serialize(obj) {
  let offset = 8;
  const msgLength = calcMsgLength(obj, null);
  const buffer = Buffer.alloc(8 + msgLength);

  buffer[0] = 1;
  buffer.writeUInt32LE(msgLength + 8, 4);

  const write = (obj) => {
    const kType = getKType(obj);
    buffer[offset] = kType;
    offset++;
    switch (kType) {
      // null
      case 101:
        buffer[offset++] = 0;
        break;
      // timestamp
      case 244: {
        const i64 = BigInt(obj.getTime() - MS_DIFF) * 1000000n;
        buffer.writeBigInt64LE(i64, offset);
        offset += 8;
      }
        break;
      // float
      case 247:
        buffer.writeDoubleLE(obj, offset);
        offset += 8;
        break;
      // bigint
      case 249:
        buffer.writeBigInt64LE(obj, offset);
        offset += 8;
        break;
      // boolean
      case 255:
        buffer[offset++] = obj ? 1 : 0;
        break;
      case 98: {
        buffer[offset++] = 0;
        buffer[offset++] = 99;

        const meta = obj[Symbol.for('meta')];
        const column = meta.c;
        // write column names (symbols)
        buffer[offset++] = 11;
        writeArray(Object.keys(obj), 11);
        buffer[offset++] = 0;
        buffer[offset++] = 0;
        buffer.writeUInt32LE(column.length, offset);
        offset += 4;
        column.forEach((c, i) => {
          let t = K_TYPE_CHAR.indexOf(meta.t[i]);
          t = t > 0 ? t : 0;
          buffer[offset++] = t;
          writeArray(obj[c], t);
        });
      }
        break;
      case 99: {
        const k = Object.keys(obj);
        const v = Object.values(obj);
        buffer[offset++] = 11;
        writeArray(k, 11);
        buffer[offset++] = 0;
        writeArray(v, 0);
      }
        break;
      case 0:
      case 1:
      case 2:
      case 6:
      case 7:
      case 9:
      case 10:
      case 11:
      case 12:
      case 14:
      case 15:
        writeArray(obj, kType);
    }
  };

  const writeArray = (obj, kType = 0) => {
    // attribute
    buffer[offset++] = 0;
    if (kType === 10) {
      // char, deal with utf8
      const length = buffer.write(obj, offset + 4);
      buffer.writeUInt32LE(length, offset);
      offset += 4 + length;
    } else {
      buffer.writeUInt32LE(obj.length, offset);
      offset += 4;
    }
    switch (kType) {
      // general list
      case 0:
        obj.forEach(o => {
          write(o);
        });
        break;
      // boolean
      case 1:
        obj.forEach(b => buffer[offset++] = b ? 1 : 0);
        break;
      // guid
      case 2:
        obj.forEach(g => {
          Buffer.from(g, 'hex').copy(buffer, offset);
          offset += 16;
        });
        break;
      // int
      case 6:
        obj.forEach(i => {
          if (i) {
            if (i === Infinity) {
              buffer.writeInt32LE(INT_POSITIVE_INFINITY, offset);
            } else if (i === -Infinity) {
              buffer.writeInt32LE(INT_NEGATIVE_INFINITY, offset);
            } else {
              buffer.writeInt32LE(i, offset);
            }
            offset += 4;
          } else {
            buffer.writeInt32LE(INT_NULL, offset);
            offset += 4;
          }
        });
        break;
      // long
      case 7:
        obj.forEach(l => {
          if (l) {
            if (l === Infinity) {
              buffer.writeBigInt64LE(LONG_POSITIVE_INFINITY, offset);
            } else if (l === -Infinity) {
              buffer.writeBigInt64LE(LONG_NEGATIVE_INFINITY, offset);
            } else {
              const bigI64 = typeof l === 'bigint' ? l : BigInt(l);
              buffer.writeBigInt64LE(bigI64, offset);
            }
          } else {
            buffer.writeBigInt64LE(LONG_NULL, offset);
          }
          offset += 8;
        });
        break;
      // float
      case 9:
        obj.forEach(f => {
          buffer.writeDoubleLE(f, offset);
          offset += 8;
        });
        break;
      // symbol
      case 11:
        obj.forEach(s => {
          buffer.write(s, offset);
          offset += s.length;
          buffer[offset++] = 0;
        });
        break;
      // timestamp
      case 12:
        obj.forEach(d => {
          if (d) {
            const ns = 1000000n * BigInt(d.getTime() - MS_DIFF);
            buffer.writeBigInt64LE(ns, offset);
          } else {
            buffer.writeBigInt64LE(LONG_NULL, offset);
          }
          offset += 8;
        });
        break;
      // date
      case 14:
        obj.forEach(d => {
          if (d) {
            const days = (d.getTime() - MS_DIFF) / MS_PER_DAY;
            // auto truncate here
            buffer.writeInt32LE(days, offset);
          } else {
            buffer.writeInt32LE(INT_NULL, offset);
          }
          offset += 4;
        });
        break;
      // datetime
      case 15:
        obj.forEach(d => {
          if (d) {
            const days = (d.getTime() - MS_DIFF) / MS_PER_DAY;
            buffer.writeDoubleLE(days, offset);
          } else {
            buffer.writeDoubleLE(NaN, offset);
          }
          offset += 8;
        });
        break;
    }
  };
  write(obj);
  return buffer;
}

const ACK = Buffer.from('010200000a0000006500', 'hex');

export default { deserialize, serialize, ACK };
