# jkdb

[![](https://img.shields.io/npm/dm/jkdb?labelColor=4a148c&color=9c27b0&style=flat)](https://www.npmjs.com/package/jkdb)

A Rust/WASM powered Node.js package to interface with kdb+/q (v2.6+).

## Installation

```
npm install jkdb
```

### Building from source

Requires [Rust](https://rustup.rs/) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/):

```bash
npm run build
```

## Quick Start

### Connect to a q Process

```javascript
import { QConnection } from "jkdb";
const q = new QConnection({ port: 1800 });
q.connect((err) => {
  if (err) throw err;
  console.log("connected");
  // send query from here
});
```

### Connect to a TLS-protected q Process

```javascript
import { QConnection } from "jkdb";
const q = new QConnection({ port: 1800, enableTLS: true });
q.connect((err) => {
  if (err) throw err;
  console.log("connected");
});
```

### Connect to a q Process with Credentials

```javascript
import { QConnection } from "jkdb";
const q = new QConnection({ port: 1800, user: "user", password: "password" });
q.connect((err) => {
  if (err) throw err;
  console.log("connected");
});
```

### Send a Sync Query

```javascript
q.sync("(+/) til 10", (err, res) => {
  if (err) throw err;
  console.log("result: ", res);
  // result: 45
});
```

### Send a Sync Function Call

```javascript
q.sync(["(*/)", [22, 27, 45]], (err, res) => {
  if (err) throw err;
  console.log("result: ", res);
  // result: 26730
});
```

### Send a Sync Function Call with Multiple Parameters

```javascript
q.sync(
  [
    "mmu",
    [1, 2, 3],
    [
      [1, 2, 3],
      [4, 5, 6],
      [7, 8, 9],
    ],
  ],
  (err, res) => {
    if (err) throw err;
    console.log("result: ", res);
    // result: 30,36,42
  }
);
```

### Send an Async Query

```javascript
q.asyn("show 99", (err) => {
  if (err) throw err;
});
```

### Send an Async Function Call

```javascript
q.asyn(["show", 99], (err) => {
  if (err) throw err;
});
```

### Subscribe

```javascript
q.on("upd", (table, data) => {
  console.log(table, data);
});

q.sync(".u.sub[`trade;`7203.T]", (err, _res) => {
  if (err) throw err;
});
```

### Close q Connection

```javascript
q.close(() => {
  console.log("closed");
});
```

### Direct IPC Usage

Use the low-level IPC codec directly:

```javascript
import { IPC } from "jkdb";

// Deserialize a kdb+ IPC message
const result = IPC.deserialize(buffer);

// With options
const result2 = IPC.deserialize(buffer, true, false, false);
//                                       ^      ^      ^
//                                  useBigInt  nano  dateMs

// Serialize a JS value to kdb+ IPC binary
const bytes = IPC.serialize({ sym: "8306.T", price: 668.2 });
```

## Connection Options

| Option             | Type    | Default     | Description                                  |
| ------------------ | ------- | ----------- | -------------------------------------------- |
| host               | string  | 'localhost' | kdb+ server hostname                         |
| port               | number  | —           | kdb+ server port (**required**)              |
| user               | string  | ''          | Username for authentication                  |
| password           | string  | ''          | Password for authentication                  |
| useBigInt          | boolean | false       | Return longs as BigInt                       |
| enableTLS          | boolean | false       | Use TLS connection                           |
| socketTimeout      | number  | 0           | Socket timeout in milliseconds               |
| socketNoDelay      | boolean | true        | Disable Nagle's algorithm                    |
| includeNanosecond  | boolean | false       | Timestamps as nanosecond-precision strings   |
| dateToMillisecond  | boolean | false       | Dates/datetimes as millisecond numbers       |

## Data Types

### Deserialization

| k type     | argument          | javascript type | k null                           | infinity | -infinity |
| ---------- | ----------------- | --------------- | -------------------------------- | -------- | --------- |
| boolean    |                   | Boolean         |                                  |          |           |
| guid       |                   | String          | 00000000000000000000000000000000 |          |           |
| byte       |                   | Number          |                                  |          |           |
| short      |                   | Number          | NaN                              | Infinity | -Infinity |
| int        |                   | Number          | NaN                              | Infinity | -Infinity |
| long       |                   | Number          | NaN                              | Infinity | -Infinity |
| long       | useBigInt         | BigInt          | NaN                              | Infinity | -Infinity |
| real       |                   | Number          | NaN                              | Infinity | -Infinity |
| float      |                   | Number          | NaN                              | Infinity | -Infinity |
| char       |                   | String          | ' '                              |          |           |
| symbol     |                   | String          | ''                               |          |           |
| timestamp  |                   | Date            | null                             | null     | null      |
| timestamp  | includeNanosecond | String          | ''                               | ''       | ''        |
| month      |                   | String          | null                             | null     | null      |
| date       |                   | Date            | null                             | null     | null      |
| date       | dateToMillisecond | Number          | NaN                              | Infinity | -Infinity |
| datetime   |                   | Date            | null                             | null     | null      |
| datetime   | dateToMillisecond | Number          | NaN                              | Infinity | -Infinity |
| timespan   |                   | String          | null                             | null     | null      |
| minute     |                   | String          | null                             | null     | null      |
| second     |                   | String          | null                             | null     | null      |
| time       |                   | String          | null                             | null     | null      |
| dict       |                   | Object          |                                  |          |           |
| list       |                   | Array           |                                  |          |           |
| table      |                   | Object          |                                  |          |           |
| lambda     |                   | String          |                                  |          |           |
| projection |                   | Array           |                                  |          |           |

### Serialization

#### Atom

| javascript type | k type    |
| --------------- | --------- |
| Boolean         | boolean   |
| Number          | float     |
| String          | chars     |
| Date            | timestamp |

#### Array

| javascript type | Symbol.for('kType') | k type    |
| --------------- | ------------------- | --------- |
| Boolean Array   | b                   | boolean   |
| String Array    | g                   | guid      |
| Number Array    | i                   | int       |
| Number Array    | j                   | long      |
| Number Array    | f                   | float     |
| String Array    | c                   | char      |
| String Array    | s                   | symbol    |
| Date Array      | p                   | timestamp |
| Date Array      | d                   | date      |
| Date Array      | z                   | datetime  |

Use `Symbol.for('kType')` to specify k type for the array, e.g.

```javascript
const ints = [99, 11, 3, 3];
ints[Symbol.for("kType")] = "j";
```

#### Object without meta

Convert to a dictionary, keys are symbols, e.g.

```javascript
const dict = { sym: "8306.T", price: 668.2 };
```

#### Object with meta

Convert to a table, e.g.

```javascript
const table = {
  sym: ["AXJO", "AXJO"],
  date: [new Date("2021-05-13"), new Date("2021-05-14")],
  open: [7000, 6000],
};
table[Symbol.for("meta")] = {
  c: ["sym", "date", "open"],
  t: ["s", "d", "f"],
};
```
