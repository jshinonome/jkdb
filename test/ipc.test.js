import IPC from '../src/ipc';

function d(hexString, useBigInt = false, includeNanosecond = false) {
  return IPC.deserialize(Buffer.from(hexString, 'hex'), useBigInt, includeNanosecond);
}

function s(obj) {
  return IPC.serialize(obj).toString('hex');
}

test('deserialize general list', () => {
  const msg = '0100000022000000000003000000f66af96300000000000000f71f85eb51b81e0940';
  const obj = ['j', 99, 3.14];
  obj[Symbol.for('kType')] = ' ';
  expect(d(msg)).toStrictEqual(obj);
});

// Number => k Float
test('deserialize/serialize general list', () => {
  const msg = '01000000280000000000030000000a00020000006a73f70000000000c05840f71f85eb51b81e0940';
  const obj = ['js', 99, 3.14];
  obj[Symbol.for('kType')] = ' ';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize function call', () => {
  const msg = '010000003d000000000004000000' + '0a001a0000002e63616c656e6461722e6765744461746554797065427953796df278200000f5373230332e54006500';
  const obj = ['.calendar.getDateTypeBySym', new Date('2022-10-04'), '7203.T', null];
  obj[Symbol.for('kType')] = ' ';
  expect(d(msg)).toStrictEqual(obj);
});

// Date => k Timestamp
test('deserialize/serialize function call', () => {
  const msg = '0100000045000000000004000000' +
    '0a001a0000002e63616c656e6461722e6765744461746554797065427953796d' +
    'f4000008fdcd67f7090a0006000000373230332e546500';
  const obj = ['.calendar.getDateTypeBySym', new Date('2022-10-04'), '7203.T', null];
  obj[Symbol.for('kType')] = ' ';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize/serialize list of list', () => {
  const msg = '0100000042000000000003000000' + '0b000200000041584a4f0041584a4f000e00020000007b1e00007c1e00000900020000002d3e05c0cc7ebb402d3e05c04c54bb40';
  const obj = [['AXJO', 'AXJO'], [new Date('2021-05-13'), new Date('2021-05-14')], [7038.799805, 6996.299805]];
  obj[0][Symbol.for('kType')] = 's';
  obj[1][Symbol.for('kType')] = 'd';
  obj[2][Symbol.for('kType')] = 'f';
  obj[Symbol.for('kType')] = ' ';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize/serialize null', () => {
  const msg = '010000000a0000006500';
  expect(d(msg)).toBe(null);
  expect(s(null)).toBe(msg);
}
);

test('deserialize/serialize boolean true', () => {
  const msg = '010000000a000000ff01';
  expect(d(msg)).toBe(true);
  expect(s(true)).toBe(msg);
});

test('deserialize/serialize boolean false', () => {
  const msg = '010000000a000000ff00';
  expect(d(msg)).toBe(false);
  expect(s(false)).toBe(msg);
});


test('deserialize/serialize boolean list', () => {
  const msg = '01000000100000000100020000000100';
  const obj = [true, false];
  obj[Symbol.for('kType')] = 'b';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize guid', () => {
  const msg = '0100000019000000feddb87915b6722c32a6cf296061671e9d';
  const obj = 'ddb87915b6722c32a6cf296061671e9d';
  expect(d(msg)).toBe(obj);
});

test('deserialize/serialize guid list', () => {
  const msg = '010000002e000000020002000000580d8c87e5570db13a19cb3a44d623b12d948578e9d679a282079df7a71f0b3b';
  const obj = ['580d8c87e5570db13a19cb3a44d623b1', '2d948578e9d679a282079df7a71f0b3b'];
  obj[Symbol.for('kType')] = 'g';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize byte', () => {
  expect(d('010000000a000000fc01')).toBe(1);
});

test('deserialize byte list', () => {
  const msg = '010000001000000004000200000001ff';
  const obj = [1, 255];
  obj[Symbol.for('kType')] = 'x';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize short', () => {
  expect(d('010000000b000000fb6300')).toBe(99);
  expect(d('010000000b000000fb0080')).toBe(NaN);
  expect(d('010000000b000000fbff7f')).toBe(Infinity);
  expect(d('010000000b000000fb0180')).toBe(-Infinity);
});

test('deserialize short list', () => {
  const msg = '010000001600000005000400000063000080ff7f0180';
  const obj = [99, NaN, Infinity, -Infinity];
  obj[Symbol.for('kType')] = 'h';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize int', () => {
  expect(d('010000000d000000fa63000000')).toBe(99);
  expect(d('010000000d000000fa00000080')).toBe(NaN);
  expect(d('010000000d000000faffffff7f')).toBe(Infinity);
  expect(d('010000000d000000fa01000080')).toBe(-Infinity);
});

test('deserialize/serialize int list', () => {
  const msg = '010000001e0000000600040000006300000000000080ffffff7f01000080';
  const obj = [99, NaN, Infinity, -Infinity];
  obj[Symbol.for('kType')] = 'i';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize long', () => {
  expect(d('0100000011000000f96300000000000000')).toBe(99);
  expect(d('0100000011000000f96300000000000000', true)).toBe(99n);
  expect(d('0100000011000000f90000000000000080')).toBe(NaN);
  expect(d('0100000011000000f9ffffffffffffff7f')).toBe(Infinity);
  expect(d('0100000011000000f90100000000000080')).toBe(-Infinity);
});

test('deserialize/serialize long list', () => {
  const msg = '010000002e000000070004000000' +
    '63000000000000000000000000000080ffffffffffffff7f0100000000000080';
  const obj = [99, NaN, Infinity, -Infinity];
  obj[Symbol.for('kType')] = 'j';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize real', () => {
  expect(d('010000000d000000f80000c642')).toBe(99);
  expect(d('010000000d000000f80000c0ff')).toBe(NaN);
  expect(d('010000000d000000f80000807f')).toBe(Infinity);
  expect(d('010000000d000000f8000080ff')).toBe(-Infinity);
});

test('deserialize real list', () => {
  const msg = '010000001e0000000800040000000000c6420000c0ff0000807f000080ff';
  const obj = [99, NaN, Infinity, -Infinity];
  obj[Symbol.for('kType')] = 'e';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize/serialize float finite', () => {
  const msg = '0100000011000000f70000000000c05840';
  const obj = 99;
  expect(d(msg)).toBe(obj);
  expect(s(obj)).toBe(msg);
});

// 000000000000f8ff or 000000000000f87f => 0n
// as writeDoubleLE write NaN 000000000000f87f, use 000000000000f87f here
test('deserialize/serialize float NaN', () => {
  const msg = '0100000011000000f7000000000000f87f';
  const obj = NaN;
  expect(d(msg)).toBe(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize/serialize float infinite', () => {
  const msg = '0100000011000000f7000000000000f07f';
  const obj = Infinity;
  expect(d(msg)).toBe(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize/serialize float -infinite', () => {
  const msg = '0100000011000000f7000000000000f0ff';
  const obj = -Infinity;
  expect(d(msg)).toBe(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize/serialize float list', () => {
  const msg = '010000002e000000090004000000' +
    '0000000000c05840000000000000f87f000000000000f07f000000000000f0ff';
  const obj = [99, NaN, Infinity, -Infinity];
  obj[Symbol.for('kType')] = 'f';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize char', () =>
  expect(d('010000000a000000f661')).toBe('a')
);

test('deserialize/serialize string', () => {
  const msg = '01000000120000000a00040000002e7a2e64';
  const obj = '.z.d';
  expect(d(msg)).toBe(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize/serialize utf8 string', () => {
  const msg = '010000001a0000000a000c000000e38282e381aee381aee38191';
  const obj = 'もののけ';
  expect(d(msg)).toBe(obj);
  expect(s(obj)).toBe(msg);
});


test('deserialize symbol', () =>
  expect(d('010000000b000000f56100')).toBe('a')
);

test('deserialize/serialize symbols', () => {
  const msg = '01000000120000000b000200000061006200';
  const obj = ['a', 'b'];
  obj[Symbol.for('kType')] = 's';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

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

test('deserialize/serialize timestamp', () => {
  const msg = '0100000011000000f400f8e10e6849f709';
  const obj = new Date('2022-10-03T14:42:56.864Z');
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize timestamp list', () => {
  const msg = '010000002e0000000c0004000000605fe30e6849f7090000000000000080ffffffffffffff7f0100000000000080';
  const obj = [new Date('2022-10-03T14:42:56.864Z'), null, null, null];
  obj[Symbol.for('kType')] = 'p';
  expect(d(msg)).toStrictEqual(obj);
});


test('deserialize timestamp list include nanosecond', () => {
  const msg = '010000002e0000000c00040000004f13ca13115eff090000000000000080ffffffffffffff7f0100000000000080';
  const obj = ['2022-10-29T22:31:32.842033999', '', '', ''];
  obj[Symbol.for('kType')] = 'p';
  expect(d(msg, false, true)).toStrictEqual(obj);
});


test('deserialize/serialize timestamp list', () => {
  const msg = '010000001e0000000c000200000000f8e10e6849f7090000000000000080';
  const obj = [new Date('2022-10-03T14:42:56.864Z'), null];
  obj[Symbol.for('kType')] = 'p';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize month', () => {
  expect(d('010000000d000000f36dffffff')).toBe('1987.10m');
  expect(d('010000000d000000f311010000')).toBe('2022.10m');
  expect(d('010000000d000000f300000080')).toBe(null);
  expect(d('010000000d000000f3ffffff7f')).toBe(null);
  expect(d('010000000d000000f301000080')).toBe(null);
});

test('deserialize month list', () => {
  const msg = '010000001e0000000d00040000001101000000000080ffffff7f01000080';
  const obj = ['2022.10m', null, null, null];
  obj[Symbol.for('kType')] = 'm';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize date', () => {
  expect(d('010000000d000000f26feeffff')).toStrictEqual(new Date('1987-09-09'));
  expect(d('010000000d000000f277200000')).toStrictEqual(new Date('2022-10-03'));
  expect(d('010000000d000000f200000080')).toBe(null);
  expect(d('010000000d000000f2ffffff7f')).toBe(null);
  expect(d('010000000d000000f201000080')).toBe(null);
});

test('deserialize date list', () => {
  const msg = d('010000001e0000000e00040000007720000000000080ffffff7f01000080');
  const obj = [new Date('2022-10-03'), null, null, null];
  obj[Symbol.for('kType')] = 'd';
  expect(msg).toStrictEqual(obj);
});

test('deserialize/serialize date list', () => {
  const msg = '01000000160000000e00020000007720000000000080';
  const obj = [new Date('2022-10-03'), null];
  obj[Symbol.for('kType')] = 'd';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});


test('deserialize datetime', () => {
  expect(d('0100000011000000f174f58bc97990b1c0')).toStrictEqual(new Date('1987-09-09T12:34:56.789Z'));
  expect(d('0100000011000000f1cccccccccc3bc040')).toStrictEqual(new Date('2022-10-03T14:24:00.000Z'));
  expect(d('0100000011000000f1000000000000f8ff')).toBe(null);
  expect(d('0100000011000000f1000000000000f07f')).toBe(null);
  expect(d('0100000011000000f1000000000000f0ff')).toBe(null);
});

test('deserialize datetime list', () => {
  const msg = '010000002e0000000f0004000000cdcccccccc3bc040000000000000f8ff000000000000f07f000000000000f0ff';
  const obj = [new Date('2022-10-03T14:24:00.000Z'), null, null, null];
  obj[Symbol.for('kType')] = 'z';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize/serialize datetime list', () => {
  const msg = '010000001e0000000f0002000000cdcccccccc3bc040000000000000f87f';
  const obj = [new Date('2022-10-03T14:24:00.000Z'), null];
  obj[Symbol.for('kType')] = 'z';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});


test('deserialize timespan', () => {
  expect(d('0100000011000000f098abebe4ccecffff')).toStrictEqual('-0D05:51:50.218577000');
  expect(d('0100000011000000f06854141b33130000')).toStrictEqual('0D05:51:50.218577000');
  expect(d('0100000011000000f00000000000000080')).toBe(null);
  expect(d('0100000011000000f0ffffffffffffff7f')).toBe(null);
  expect(d('0100000011000000f00100000000000080')).toBe(null);
});

test('deserialize timespan list', () => {
  const msg = '010000002e0000001000040000006854141b331300000000000000000080ffffffffffffff7f0100000000000080';
  const obj = ['0D05:51:50.218577000', null, null, null];
  obj[Symbol.for('kType')] = 'n';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize minute', () => {
  expect(d('010000000d000000eff7ffffff')).toStrictEqual('-00:09');
  expect(d('010000000d000000efadfcffff')).toStrictEqual('-14:11');
  expect(d('010000000d000000ef53030000')).toStrictEqual('14:11');
  expect(d('010000000d000000ef00000080')).toBe(null);
  expect(d('010000000d000000efffffff7f')).toBe(null);
  expect(d('010000000d000000ef01000080')).toBe(null);
});

test('deserialize minute list', () => {
  const msg = '010000001e0000001100040000005303000000000080ffffff7f01000080';
  const obj = ['14:11', null, null, null];
  obj[Symbol.for('kType')] = 'u';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize second', () => {
  expect(d('010000000d000000eef7ffffff')).toStrictEqual('-00:00:09');
  expect(d('010000000d000000ee5b38ffff')).toStrictEqual('-14:11:49');
  expect(d('010000000d000000eea5c70000')).toStrictEqual('14:11:49');
  expect(d('010000000d000000ee00000080')).toBe(null);
  expect(d('010000000d000000eeffffff7f')).toBe(null);
  expect(d('010000000d000000ee01000080')).toBe(null);
});

test('deserialize second list', () => {
  const msg = '010000001e000000120004000000a5c7000000000080ffffff7f01000080';
  const obj = ['14:11:49', null, null, null];
  obj[Symbol.for('kType')] = 'v';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize time', () => {
  expect(d('010000000d000000edf7ffffff')).toStrictEqual('-00:00:00.009');
  expect(d('010000000d000000eddc20f4fc')).toStrictEqual('-14:11:49.668');
  expect(d('010000000d000000ed24df0b03')).toStrictEqual('14:11:49.668');
  expect(d('010000000d000000ed00000080')).toBe(null);
  expect(d('010000000d000000edffffff7f')).toBe(null);
  expect(d('010000000d000000ed01000080')).toBe(null);
});

test('deserialize time list', () => {
  const msg = '010000001e00000013000400000024df0b0300000080ffffff7f01000080';
  const obj = ['14:11:49.668', null, null, null];
  obj[Symbol.for('kType')] = 't';
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize dictionary', () => {
  const msg = '0100000029000000630b00020000006100620007000200000001000000000000000200000000000000';
  const obj = { 'a': 1, 'b': 2 };
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize/serialize dictionary', () => {
  const msg = '0100000034000000' +
    '630b000200000073796d00707269636500' +
    '0000020000000a0006000000383330362e54f79a99999999e18440';
  const obj = { 'sym': '8306.T', 'price': 668.2 };
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize dictionary with table value', () => {
  const msg = '010000005900000063' + '0b00010000006f757470757400' + '6200630b000300000077006800627974657300000003000000070001000000640000000000000007000100000064000000000000000700010000006400000000000000';
  const table = {
    'w': 100,
    'h': 100,
    'bytes': 100,
  };
  const obj = { 'output': table };
  expect(d(msg)).toStrictEqual(obj);
});

test('deserialize/serialize flip table', () => {
  const msg = '010000005700000063' +
    '0b000300000073796d0064617465006f70656e00' + '000003000000' +
    '0b000200000041584a4f0041584a4f00' + '0e00020000007b1e00007c1e0000' +
    '090002000000000000000058bb40000000000070b740';
  const obj = {
    'sym': ['AXJO', 'AXJO'],
    'date': [new Date('2021-05-13'), new Date('2021-05-14')],
    'open': [7000, 6000]
  };
  obj['sym'][Symbol.for('kType')] = 's';
  obj['date'][Symbol.for('kType')] = 'd';
  obj['open'][Symbol.for('kType')] = 'f';
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize/serialize table', () => {
  const msg = '010000005900000062' + '0063' +
    '0b000300000073796d0064617465006f70656e00' + '000003000000' +
    '0b000200000041584a4f0041584a4f00' + '0e00020000007b1e00007c1e0000' +
    '090002000000000000000058bb40000000000070b740';
  const obj = {
    'sym': ['AXJO', 'AXJO'],
    'date': [new Date('2021-05-13'), new Date('2021-05-14')],
    'open': [7000, 6000]
  };
  obj[Symbol.for('meta')] = {
    c: ['sym', 'date', 'open'],
    t: ['s', 'd', 'f']
  };
  expect(d(msg)).toStrictEqual(obj);
  expect(s(obj)).toBe(msg);
});

test('deserialize keyed table', () => {
  const msg = '01000000690000' +
    '0063' +
    '6200630b000100000073796d000000010000000b000200000041584a4f0041584a4f00' + '6200630b000200000064617465006f70656e00' +
    '0000020000000e00020000007b1e00007c1e0000090002000000000000000058bb40000000000070b740';
  const obj = {
    'sym': ['AXJO', 'AXJO'],
    'date': [new Date('2021-05-13'), new Date('2021-05-14')],
    'open': [7000, 6000]
  };
  obj[Symbol.for('meta')] = {
    c: ['sym', 'date', 'open'],
    t: ['s', 'd', 'f']
  };
  obj[Symbol.for('keys')] = ['sym'];
  expect(d(msg)).toStrictEqual(obj);
});

test('decompression', () => {
  const msg = '0110010026000000de070000000100d00700000101ff00ff00ff00ff00ff00ff00ff00ff00c5';
  const obj = Array.from({ length: 2000 }, (_v, _k) => true);
  obj[Symbol.for('kType')] = 'b';
  expect(d(msg)).toStrictEqual(obj);
});
