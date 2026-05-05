/**
 * WASM IPC codec test.
 *
 * Wraps the WASM ipc_deserialize / ipc_serialize to match the
 * original JS IPC test interface, so the same test cases apply.
 */
import { ipc_deserialize, ipc_serialize } from '../pkg/jkdb.js';

function d(hexString, useBigInt = false, includeNanosecond = false, dateToMillisecond = false) {
  const buf = Buffer.from(hexString, 'hex');
  return ipc_deserialize(new Uint8Array(buf), useBigInt, includeNanosecond, dateToMillisecond);
}

// ---- Atoms ----

test('deserialize null', () => {
  expect(d('010000000a0000006500')).toBe(null);
});

test('deserialize boolean true', () => {
  expect(d('010000000a000000ff01')).toBe(true);
});

test('deserialize boolean false', () => {
  expect(d('010000000a000000ff00')).toBe(false);
});

test('deserialize byte', () => {
  expect(d('010000000a000000fc01')).toBe(1);
});

test('deserialize short', () => {
  expect(d('010000000b000000fb6300')).toBe(99);
  // Null/inf sentinels
  expect(d('010000000b000000fb0080')).toBe(NaN);
  expect(d('010000000b000000fbff7f')).toBe(Infinity);
  expect(d('010000000b000000fb0180')).toBe(-Infinity);
});

test('deserialize int', () => {
  expect(d('010000000d000000fa63000000')).toBe(99);
  expect(d('010000000d000000fa00000080')).toBe(NaN);
  expect(d('010000000d000000faffffff7f')).toBe(Infinity);
  expect(d('010000000d000000fa01000080')).toBe(-Infinity);
});

test('deserialize long', () => {
  expect(d('0100000011000000f96300000000000000')).toBe(99);
  expect(d('0100000011000000f96300000000000000', true)).toBe(99n);
  expect(d('0100000011000000f90000000000000080')).toBe(NaN);
  expect(d('0100000011000000f9ffffffffffffff7f')).toBe(Infinity);
  expect(d('0100000011000000f90100000000000080')).toBe(-Infinity);
});

test('deserialize float', () => {
  expect(d('0100000011000000f70000000000c05840')).toBe(99);
});

test('deserialize char', () =>
  expect(d('010000000a000000f661')).toBe('a')
);

test('deserialize symbol', () =>
  expect(d('010000000b000000f56100')).toBe('a')
);

test('deserialize string', () => {
  expect(d('01000000120000000a00040000002e7a2e64')).toBe('.z.d');
});

test('deserialize guid', () => {
  expect(d('0100000019000000feddb87915b6722c32a6cf296061671e9d')).toBe('ddb87915b6722c32a6cf296061671e9d');
});

// ---- Timestamps ----

test('deserialize timestamp', () => {
  expect(d('0100000011000000f4605fe30e6849f709')).toStrictEqual(new Date('2022-10-03T14:42:56.864Z'));
  expect(d('0100000011000000f40000000000000080')).toBe(null);
  expect(d('0100000011000000f4ffffffffffffff7f')).toBe(null);
  expect(d('0100000011000000f40100000000000080')).toBe(null);
});

test('deserialize timestamp include nanosecond', () => {
  expect(d('0100000011000000f44f13ca13115eff09', false, true)).toStrictEqual('2022-10-29T22:31:32.842033999');
  expect(d('0100000011000000f40000000000000080', false, true)).toBe('');
  expect(d('0100000011000000f4ffffffffffffff7f', false, true)).toBe('');
  expect(d('0100000011000000f40100000000000080', false, true)).toBe('');
});

// ---- Temporal ----

test('deserialize month', () => {
  expect(d('010000000d000000f36dffffff')).toBe('1987.10m');
  expect(d('010000000d000000f311010000')).toBe('2022.10m');
  expect(d('010000000d000000f300000080')).toBe(null);
});

test('deserialize date', () => {
  expect(d('010000000d000000f277200000')).toStrictEqual(new Date('2022-10-03'));
  expect(d('010000000d000000f200000080')).toBe(null);
});

test('deserialize date to millisecond', () => {
  expect(d('010000000d000000f277200000', false, false, true)).toBe(1664755200000);
  expect(d('010000000d000000f200000080', false, false, true)).toBe(NaN);
  expect(d('010000000d000000f2ffffff7f', false, false, true)).toBe(Infinity);
  expect(d('010000000d000000f201000080', false, false, true)).toBe(-Infinity);
});

test('deserialize timespan', () => {
  expect(d('0100000011000000f06854141b33130000')).toStrictEqual('0D05:51:50.218577000');
  expect(d('0100000011000000f098abebe4ccecffff')).toStrictEqual('-0D05:51:50.218577000');
  expect(d('0100000011000000f00000000000000080')).toBe(null);
});

test('deserialize minute', () => {
  expect(d('010000000d000000ef53030000')).toStrictEqual('14:11');
  expect(d('010000000d000000ef00000080')).toBe(null);
});

test('deserialize second', () => {
  expect(d('010000000d000000eea5c70000')).toStrictEqual('14:11:49');
  expect(d('010000000d000000ee00000080')).toBe(null);
});

test('deserialize time', () => {
  expect(d('010000000d000000ed24df0b03')).toStrictEqual('14:11:49.668');
  expect(d('010000000d000000ed00000080')).toBe(null);
});

// ---- Lists ----

test('deserialize boolean list', () => {
  const obj = d('01000000100000000100020000000100');
  expect(Array.from(obj)).toEqual([true, false]);
  expect(obj[Symbol.for('kType')]).toBe('b');
});

test('deserialize int list', () => {
  const obj = d('010000001e0000000600040000006300000000000080ffffff7f01000080');
  expect(Array.from(obj)).toEqual([99, NaN, Infinity, -Infinity]);
  expect(obj[Symbol.for('kType')]).toBe('i');
});

test('deserialize long list', () => {
  const msg = '010000002e000000070004000000' +
    '63000000000000000000000000000080ffffffffffffff7f0100000000000080';
  const obj = d(msg);
  expect(Array.from(obj)).toEqual([99, NaN, Infinity, -Infinity]);
  expect(obj[Symbol.for('kType')]).toBe('j');
});

test('deserialize float list', () => {
  const msg = '010000002e000000090004000000' +
    '0000000000c05840000000000000f87f000000000000f07f000000000000f0ff';
  const obj = d(msg);
  expect(Array.from(obj)).toEqual([99, NaN, Infinity, -Infinity]);
  expect(obj[Symbol.for('kType')]).toBe('f');
});

test('deserialize symbols', () => {
  const obj = d('01000000120000000b000200000061006200');
  expect(Array.from(obj)).toEqual(['a', 'b']);
  expect(obj[Symbol.for('kType')]).toBe('s');
});

// ---- Compound types ----

test('deserialize dictionary', () => {
  const msg = '0100000029000000630b00020000006100620007000200000001000000000000000200000000000000';
  expect(d(msg)).toStrictEqual({ 'a': 1, 'b': 2 });
});

test('deserialize table', () => {
  const msg = '010000005900000062' + '0063' +
    '0b000300000073796d0064617465006f70656e00' + '000003000000' +
    '0b000200000041584a4f0041584a4f00' + '0e00020000007b1e00007c1e0000' +
    '090002000000000000000058bb40000000000070b740';
  const obj = d(msg);
  expect(obj).toHaveProperty('sym');
  expect(obj).toHaveProperty('date');
  expect(obj).toHaveProperty('open');
  expect(obj[Symbol.for('meta')]).toBeDefined();
  expect(obj[Symbol.for('meta')].c).toStrictEqual(['sym', 'date', 'open']);
  expect(obj[Symbol.for('meta')].t).toStrictEqual(['s', 'd', 'f']);
});

test('deserialize keyed table', () => {
  const msg = '01000000690000' +
    '0063' +
    '6200630b000100000073796d000000010000000b000200000041584a4f0041584a4f00' +
    '6200630b000200000064617465006f70656e00' +
    '0000020000000e00020000007b1e00007c1e0000090002000000000000000058bb40000000000070b740';
  const obj = d(msg);
  expect(obj[Symbol.for('keys')]).toStrictEqual(['sym']);
  expect(obj[Symbol.for('meta')]).toBeDefined();
});

test('deserialize lambda', () => {
  expect(d('010000001500000064000a00050000007b782b797d')).toBe('{x+y}');
});

test('deserialize unary primitive', () => {
  expect(d('010000000a00000065ff')).toStrictEqual('::');
});

test('deserialize operator', () => {
  expect(d('010000000a0000006619')).toStrictEqual('like');
});

test('deserialize iterator', () => {
  expect(d('010000000a0000006700')).toStrictEqual('\'');
});

// ---- Decompression ----

test('decompression', () => {
  const msg = '0110010026000000de070000000100d00700000101ff00ff00ff00ff00ff00ff00ff00ff00c5';
  const obj = d(msg);
  expect(obj.length).toBe(2000);
  expect(obj.every(v => v === true)).toBe(true);
  expect(obj[Symbol.for('kType')]).toBe('b');
});

// ---- Serialization ----

function s(obj) {
  return Buffer.from(ipc_serialize(obj)).toString('hex');
}

test('serialize null', () => {
  expect(s(null)).toBe('010000000a0000006500');
});

test('serialize boolean', () => {
  expect(s(true)).toBe('010000000a000000ff01');
  expect(s(false)).toBe('010000000a000000ff00');
});

test('serialize string', () => {
  expect(s('.z.d')).toBe('01000000120000000a00040000002e7a2e64');
});

test('serialize float', () => {
  expect(s(99)).toBe('0100000011000000f70000000000c05840');
});

test('serialize bigint', () => {
  expect(s(1n)).toBe('0100000011000000f90100000000000000');
});

test('serialize date', () => {
  const msg = s(new Date('2022-10-03T14:42:56.864Z'));
  expect(msg).toBe('0100000011000000f400f8e10e6849f709');
});

test('serialize general list (function call)', () => {
  const obj = ['sum til 10'];
  const hex = s(obj);
  // Should serialize as general list with one CharList element
  expect(hex.length).toBeGreaterThan(0);
  // Round-trip: deserialize should give back a general list
  const result = d(hex);
  expect(result[0]).toBe('sum til 10');
});

test('serialize symbols', () => {
  const obj = ['a', 'b'];
  obj[Symbol.for('kType')] = 's';
  expect(s(obj)).toBe('01000000120000000b000200000061006200');
});

test('serialize dict', () => {
  const obj = { 'sym': '8306.T', 'price': 668.2 };
  expect(s(obj)).toBe('0100000034000000' +
    '630b000200000073796d00707269636500' +
    '0000020000000a0006000000383330362e54f79a99999999e18440');
});

