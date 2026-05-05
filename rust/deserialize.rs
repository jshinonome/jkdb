/// KDB+ IPC binary deserialization: buffer → KObject
use crate::decompress::decompress;
use crate::error::KError;
use crate::k_types::*;
use crate::kobject::KObject;

pub struct DeserializeOptions {
    pub use_big_int: bool,
    pub include_nanosecond: bool,
    pub date_to_millisecond: bool,
}

impl Default for DeserializeOptions {
    fn default() -> Self {
        Self {
            use_big_int: false,
            include_nanosecond: false,
            date_to_millisecond: false,
        }
    }
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn u8(&mut self) -> u8 {
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }
    fn i16le(&mut self) -> i16 {
        let v = i16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        v
    }
    fn i32le(&mut self) -> i32 {
        let b = &self.buf[self.pos..self.pos + 4];
        self.pos += 4;
        i32::from_le_bytes(b.try_into().unwrap())
    }
    fn i64le(&mut self) -> i64 {
        let b = &self.buf[self.pos..self.pos + 8];
        self.pos += 8;
        i64::from_le_bytes(b.try_into().unwrap())
    }
    fn f32le(&mut self) -> f32 {
        let b = &self.buf[self.pos..self.pos + 4];
        self.pos += 4;
        f32::from_le_bytes(b.try_into().unwrap())
    }
    fn f64le(&mut self) -> f64 {
        let b = &self.buf[self.pos..self.pos + 8];
        self.pos += 8;
        f64::from_le_bytes(b.try_into().unwrap())
    }

    /// Bulk-read `n` elements of type `T` from the buffer via pointer cast.
    /// Safe on LE targets (WASM, x86) since kdb+ IPC is always little-endian.
    #[inline]
    unsafe fn read_slice<T: Copy>(&mut self, n: usize) -> Vec<T> {
        let byte_len = n * core::mem::size_of::<T>();
        let src = &self.buf[self.pos..self.pos + byte_len];
        let mut v = Vec::<T>::with_capacity(n);
        unsafe {
            core::ptr::copy_nonoverlapping(src.as_ptr(), v.as_mut_ptr() as *mut u8, byte_len);
            v.set_len(n);
        };
        self.pos += byte_len;
        v
    }

    fn read_atom(&mut self, k_type: u8) -> Result<KObject, KError> {
        match k_type {
            255 => Ok(KObject::Boolean(self.u8() == 1)),
            254 => {
                let mut g = [0u8; 16];
                g.copy_from_slice(&self.buf[self.pos..self.pos + 16]);
                self.pos += 16;
                Ok(KObject::Guid(g))
            }
            252 => Ok(KObject::Byte(self.u8())),
            251 => Ok(KObject::Short(self.i16le())),
            250 => Ok(KObject::Int(self.i32le())),
            249 => Ok(KObject::Long(self.i64le())),
            248 => Ok(KObject::Real(self.f32le())),
            247 => Ok(KObject::Float(self.f64le())),
            246 => Ok(KObject::Char(self.u8() as char)),
            245 => {
                let end = self.buf[self.pos..].iter().position(|&b| b == 0).unwrap() + self.pos;
                let s = String::from_utf8_lossy(&self.buf[self.pos..end]).to_string();
                self.pos = end + 1;
                Ok(KObject::Symbol(s))
            }
            244 => {
                let ns = self.i64le();
                Ok(KObject::Timestamp(ns))
            }
            243 => {
                let v = self.i32le();
                Ok(KObject::Month(v))
            }
            242 => {
                let v = self.i32le();
                Ok(KObject::Date(v))
            }
            241 => {
                let v = self.f64le();
                Ok(KObject::DateTime(v))
            }
            240 => {
                let ns = self.i64le();
                Ok(KObject::Timespan(ns))
            }
            239 => {
                let v = self.i32le();
                Ok(KObject::Minute(v))
            }
            238 => {
                let v = self.i32le();
                Ok(KObject::Second(v))
            }
            237 => {
                let v = self.i32le();
                Ok(KObject::Time(v))
            }
            _ => Err(KError::UnsupportedKType(k_type)),
        }
    }

    fn read_array(&mut self, k_type: u8) -> Result<KObject, KError> {
        self.pos += 1; // skip attribute
        let n = self.i32le() as usize;

        match k_type {
            // Variable-length element types — must iterate
            0 => {
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    v.push(self.read(false)?);
                }
                Ok(KObject::List(v))
            }
            10 => {
                let s = String::from_utf8_lossy(&self.buf[self.pos..self.pos + n]).to_string();
                self.pos += n;
                Ok(KObject::CharList(s))
            }
            11 => {
                let v: Vec<String> = (0..n)
                    .map(|_| {
                        let end =
                            self.buf[self.pos..].iter().position(|&b| b == 0).unwrap() + self.pos;
                        let s = String::from_utf8_lossy(&self.buf[self.pos..end]).to_string();
                        self.pos = end + 1;
                        s
                    })
                    .collect();
                Ok(KObject::SymbolList(v))
            }

            // Fixed-size types — bulk read via pointer cast
            // SAFETY: kdb+ IPC is little-endian, WASM and x86 are also LE.
            // The buffer slice is guaranteed to have n * size_of::<T> bytes available.
            1 => {
                let bytes = self.buf[self.pos..self.pos + n].to_vec();
                self.pos += n;
                Ok(KObject::BooleanList(
                    bytes.into_iter().map(|b| b == 1).collect(),
                ))
            }
            2 => {
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    let mut g = [0u8; 16];
                    g.copy_from_slice(&self.buf[self.pos..self.pos + 16]);
                    self.pos += 16;
                    v.push(g);
                }
                Ok(KObject::GuidList(v))
            }
            4 => {
                let v = self.buf[self.pos..self.pos + n].to_vec();
                self.pos += n;
                Ok(KObject::ByteList(v))
            }
            5 => {
                let v = unsafe { self.read_slice::<i16>(n) };
                Ok(KObject::ShortList(v))
            }
            6 => {
                let v = unsafe { self.read_slice::<i32>(n) };
                Ok(KObject::IntList(v))
            }
            7 => {
                let v = unsafe { self.read_slice::<i64>(n) };
                Ok(KObject::LongList(v))
            }
            8 => {
                let v = unsafe { self.read_slice::<f32>(n) };
                Ok(KObject::RealList(v))
            }
            9 => {
                let v = unsafe { self.read_slice::<f64>(n) };
                Ok(KObject::FloatList(v))
            }
            12 => {
                let v = unsafe { self.read_slice::<i64>(n) };
                Ok(KObject::TimestampList(v))
            }
            13 => {
                let v = unsafe { self.read_slice::<i32>(n) };
                Ok(KObject::MonthList(v))
            }
            14 => {
                let v = unsafe { self.read_slice::<i32>(n) };
                Ok(KObject::DateList(v))
            }
            15 => {
                let v = unsafe { self.read_slice::<f64>(n) };
                Ok(KObject::DateTimeList(v))
            }
            16 => {
                let v = unsafe { self.read_slice::<i64>(n) };
                Ok(KObject::TimespanList(v))
            }
            17 => {
                let v = unsafe { self.read_slice::<i32>(n) };
                Ok(KObject::MinuteList(v))
            }
            18 => {
                let v = unsafe { self.read_slice::<i32>(n) };
                Ok(KObject::SecondList(v))
            }
            19 => {
                let v = unsafe { self.read_slice::<i32>(n) };
                Ok(KObject::TimeList(v))
            }
            _ => Err(KError::UnsupportedKList(k_type)),
        }
    }

    fn read(&mut self, flip_table: bool) -> Result<KObject, KError> {
        let k_type = self.u8();

        // Error
        if k_type == 128 {
            let msg_obj = self.read_atom(245)?;
            if let KObject::Symbol(s) = msg_obj {
                return Err(KError::InvalidMessage(s));
            }
        }

        // Atoms (237..=255)
        if (237..=255).contains(&k_type) {
            return self.read_atom(k_type);
        }

        // Lists (0..=19)
        if k_type <= 19 {
            return self.read_array(k_type);
        }

        // Dict / keyed table (99)
        if k_type == 99 {
            let is_key_table = self.buf[self.pos] == 98;
            let k = self.read(false)?;
            let has_meta = matches!(&k, KObject::Table { .. });
            let v = if has_meta {
                self.read(false)?
            } else {
                self.read(true)?
            };

            if is_key_table {
                // Both k and v are guaranteed to be Tables when is_key_table
                match (k, v) {
                    (
                        KObject::Table {
                            columns: kc,
                            values: kv,
                            col_types: kt,
                        },
                        KObject::Table {
                            columns: vc,
                            values: vv,
                            col_types: vt,
                        },
                    ) => {
                        let mut all_cols = kc.clone();
                        all_cols.extend(vc);
                        let mut all_vals = kv;
                        all_vals.extend(vv);
                        let mut all_types = kt;
                        all_types.extend(vt);
                        return Ok(KObject::KeyedTable {
                            key_columns: kc,
                            columns: all_cols,
                            values: all_vals,
                            col_types: all_types,
                        });
                    }
                    (k, v) => {
                        return Ok(KObject::Dict {
                            keys: Box::new(k),
                            values: Box::new(v),
                        });
                    }
                }
            }
            return Ok(KObject::Dict {
                keys: Box::new(k),
                values: Box::new(v),
            });
        }

        // Table (98)
        if k_type == 98 {
            self.pos += 3; // skip attr, dict kType (99), sym kType (11)
            let cols = if let KObject::SymbolList(v) = self.read_array(11)? {
                v
            } else {
                return Err(KError::UnsupportedKType(98));
            };
            self.pos += 6; // skip mixed list header (kType=0, attr, length)
            let mut values = Vec::with_capacity(cols.len());
            let mut col_types = Vec::with_capacity(cols.len());
            for _ in 0..cols.len() {
                let ct = self.u8();
                col_types.push(if ct < K_TYPE_CHAR.len() as u8 {
                    K_TYPE_CHAR[ct as usize]
                } else {
                    b' '
                });
                values.push(self.read_array(ct)?);
            }
            if flip_table {
                // Convert to list of row dicts
                let row_count = match &values[0] {
                    KObject::SymbolList(v) => v.len(),
                    KObject::IntList(v) => v.len(),
                    KObject::FloatList(v) => v.len(),
                    KObject::LongList(v) => v.len(),
                    KObject::BooleanList(v) => v.len(),
                    KObject::DateList(v) => v.len(),
                    KObject::TimestampList(v) => v.len(),
                    KObject::DateTimeList(v) => v.len(),
                    KObject::RealList(v) => v.len(),
                    KObject::ShortList(v) => v.len(),
                    KObject::ByteList(v) => v.len(),
                    KObject::GuidList(v) => v.len(),
                    KObject::MonthList(v) => v.len(),
                    KObject::TimespanList(v) => v.len(),
                    KObject::MinuteList(v) => v.len(),
                    KObject::SecondList(v) => v.len(),
                    KObject::TimeList(v) => v.len(),
                    KObject::List(v) => v.len(),
                    _ => 0,
                };
                let mut rows = Vec::with_capacity(row_count);
                for i in 0..row_count {
                    let mut row_keys = Vec::with_capacity(cols.len());
                    let mut row_vals = Vec::with_capacity(cols.len());
                    for (ci, col_val) in values.iter().enumerate() {
                        row_keys.push(cols[ci].clone());
                        row_vals.push(get_element(col_val, i));
                    }
                    rows.push(KObject::Dict {
                        keys: Box::new(KObject::SymbolList(row_keys)),
                        values: Box::new(KObject::List(row_vals)),
                    });
                }
                return Ok(KObject::FlipRows(rows));
            }
            return Ok(KObject::Table {
                columns: cols,
                values,
                col_types,
            });
        }

        // Lambda (100)
        if k_type == 100 {
            if self.u8() > 0 {
                self.pos += 1;
            }
            self.pos += 1;
            return self.read_array(10);
        }

        // Unary primitive (101)
        if k_type == 101 {
            let code = self.u8();
            if code == 0 {
                return Ok(KObject::Null);
            }
            return Ok(KObject::UnaryPrimitive(code));
        }

        // Operator (102)
        if k_type == 102 {
            return Ok(KObject::Operator(self.u8()));
        }

        // Iterator (103)
        if k_type == 103 {
            return Ok(KObject::Iterator(self.u8()));
        }

        // Projection (104)
        if k_type == 104 {
            self.pos -= 1; // no attribute to skip, move back
            return self.read_array(0);
        }

        Err(KError::UnsupportedKType(k_type))
    }
}

/// Extract element at index from a typed list
pub fn get_element(list: &KObject, i: usize) -> KObject {
    match list {
        KObject::BooleanList(v) => KObject::Boolean(v[i]),
        KObject::GuidList(v) => KObject::Guid(v[i]),
        KObject::ByteList(v) => KObject::Byte(v[i]),
        KObject::ShortList(v) => KObject::Short(v[i]),
        KObject::IntList(v) => KObject::Int(v[i]),
        KObject::LongList(v) => KObject::Long(v[i]),
        KObject::RealList(v) => KObject::Real(v[i]),
        KObject::FloatList(v) => KObject::Float(v[i]),
        KObject::SymbolList(v) => KObject::Symbol(v[i].clone()),
        KObject::TimestampList(v) => KObject::Timestamp(v[i]),
        KObject::MonthList(v) => KObject::Month(v[i]),
        KObject::DateList(v) => KObject::Date(v[i]),
        KObject::DateTimeList(v) => KObject::DateTime(v[i]),
        KObject::TimespanList(v) => KObject::Timespan(v[i]),
        KObject::MinuteList(v) => KObject::Minute(v[i]),
        KObject::SecondList(v) => KObject::Second(v[i]),
        KObject::TimeList(v) => KObject::Time(v[i]),
        KObject::List(v) => v[i].clone(),
        _ => KObject::Null,
    }
}

/// Deserialize a kdb+ IPC message buffer into a KObject.
pub fn deserialize(buffer: &[u8]) -> Result<KObject, KError> {
    let buf = if buffer.len() > 2 && buffer[2] == 1 {
        decompress(buffer)
    } else {
        buffer.to_vec()
    };
    let mut reader = Reader {
        buf: &buf,
        pos: 8,
    };
    reader.read(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(hex: &str) -> KObject {
        let buf = hex::decode(hex).unwrap();
        deserialize(&buf).unwrap()
    }

    #[test]
    fn test_null() {
        assert_eq!(d("010000000a0000006500"), KObject::Null);
    }

    #[test]
    fn test_boolean_true() {
        assert_eq!(d("010000000a000000ff01"), KObject::Boolean(true));
    }

    #[test]
    fn test_boolean_false() {
        assert_eq!(d("010000000a000000ff00"), KObject::Boolean(false));
    }

    #[test]
    fn test_boolean_list() {
        assert_eq!(
            d("01000000100000000100020000000100"),
            KObject::BooleanList(vec![true, false])
        );
    }

    #[test]
    fn test_guid() {
        let obj = d("0100000019000000feddb87915b6722c32a6cf296061671e9d");
        let expected = hex::decode("ddb87915b6722c32a6cf296061671e9d").unwrap();
        let mut g = [0u8; 16];
        g.copy_from_slice(&expected);
        assert_eq!(obj, KObject::Guid(g));
    }

    #[test]
    fn test_byte() {
        assert_eq!(d("010000000a000000fc01"), KObject::Byte(1));
    }

    #[test]
    fn test_short() {
        assert_eq!(d("010000000b000000fb6300"), KObject::Short(99));
        assert_eq!(d("010000000b000000fb0080"), KObject::Short(SHORT_NULL));
    }

    #[test]
    fn test_int() {
        assert_eq!(d("010000000d000000fa63000000"), KObject::Int(99));
    }

    #[test]
    fn test_long() {
        assert_eq!(d("0100000011000000f96300000000000000"), KObject::Long(99));
    }

    #[test]
    fn test_float() {
        if let KObject::Float(v) = d("0100000011000000f70000000000c05840") {
            assert!((v - 99.0).abs() < 1e-10);
        } else {
            panic!("expected Float");
        }
    }

    #[test]
    fn test_char() {
        assert_eq!(d("010000000a000000f661"), KObject::Char('a'));
    }

    #[test]
    fn test_symbol() {
        assert_eq!(d("010000000b000000f56100"), KObject::Symbol("a".into()));
    }

    #[test]
    fn test_string() {
        assert_eq!(
            d("01000000120000000a00040000002e7a2e64"),
            KObject::CharList(".z.d".into())
        );
    }

    #[test]
    fn test_symbols() {
        assert_eq!(
            d("01000000120000000b000200000061006200"),
            KObject::SymbolList(vec!["a".into(), "b".into()])
        );
    }

    #[test]
    fn test_int_list() {
        assert_eq!(
            d("010000001e0000000600040000006300000000000080ffffff7f01000080"),
            KObject::IntList(vec![99, INT_NULL, INT_POS_INF, INT_NEG_INF])
        );
    }

    #[test]
    fn test_long_list() {
        let obj = d(
            "010000002e00000007000400000063000000000000000000000000000080ffffffffffffff7f0100000000000080",
        );
        assert_eq!(
            obj,
            KObject::LongList(vec![99, LONG_NULL, LONG_POS_INF, LONG_NEG_INF])
        );
    }

    #[test]
    fn test_timestamp() {
        // bytes at offset 8: 60 5f e3 0e 68 49 f7 09 → i64 LE
        let expected_ns = i64::from_le_bytes([0x60, 0x5f, 0xe3, 0x0e, 0x68, 0x49, 0xf7, 0x09]);
        if let KObject::Timestamp(ns) = d("0100000011000000f4605fe30e6849f709") {
            assert_eq!(ns, expected_ns);
        } else {
            panic!("expected Timestamp");
        }
    }

    #[test]
    fn test_date() {
        assert_eq!(d("010000000d000000f277200000"), KObject::Date(8311));
    }

    #[test]
    fn test_month() {
        assert_eq!(d("010000000d000000f311010000"), KObject::Month(273));
    }

    #[test]
    fn test_timespan() {
        if let KObject::Timespan(ns) = d("0100000011000000f06854141b33130000") {
            assert_eq!(ns, 21_110_218_577_000);
        } else {
            panic!("expected Timespan");
        }
    }

    #[test]
    fn test_unary_primitive() {
        assert_eq!(d("010000000a00000065ff"), KObject::UnaryPrimitive(255));
    }

    #[test]
    fn test_operator() {
        assert_eq!(d("010000000a0000006619"), KObject::Operator(25));
    }

    #[test]
    fn test_iterator() {
        assert_eq!(d("010000000a0000006700"), KObject::Iterator(0));
    }

    #[test]
    fn test_lambda() {
        assert_eq!(
            d("010000001500000064000a00050000007b782b797d"),
            KObject::CharList("{x+y}".into())
        );
    }

    #[test]
    fn test_dict() {
        let obj =
            d("0100000029000000630b00020000006100620007000200000001000000000000000200000000000000");
        assert_eq!(
            obj,
            KObject::Dict {
                keys: Box::new(KObject::SymbolList(vec!["a".into(), "b".into()])),
                values: Box::new(KObject::LongList(vec![1, 2])),
            }
        );
    }

    #[test]
    fn test_table() {
        let obj = d(
            "01000000590000006200630b000300000073796d0064617465006f70656e000000030000000b000200000041584a4f0041584a4f000e00020000007b1e00007c1e0000090002000000000000000058bb40000000000070b740",
        );
        if let KObject::Table {
            columns,
            values,
            col_types,
        } = obj
        {
            assert_eq!(columns, vec!["sym", "date", "open"]);
            assert_eq!(col_types, vec![b's', b'd', b'f']);
            assert_eq!(values.len(), 3);
        } else {
            panic!("expected Table");
        }
    }

    #[test]
    fn test_decompression() {
        let obj = d("0110010026000000de070000000100d00700000101ff00ff00ff00ff00ff00ff00ff00ff00c5");
        if let KObject::BooleanList(v) = obj {
            assert_eq!(v.len(), 2000);
            assert!(v.iter().all(|&b| b));
        } else {
            panic!("expected BooleanList");
        }
    }
}
