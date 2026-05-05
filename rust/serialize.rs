/// KDB+ IPC binary serialization: KObject → buffer
use crate::error::KError;
use crate::k_types::*;
use crate::kobject::KObject;

/// Calculate serialized byte length of a KObject (excluding 8-byte header).
fn calc_len(obj: &KObject) -> Result<usize, KError> {
    match obj {
        KObject::Null => Ok(2), // kType + code
        KObject::Boolean(_) => Ok(2),
        KObject::Float(_) => Ok(9),
        KObject::Long(_) => Ok(9),
        KObject::Timestamp(_) => Ok(9),
        KObject::Char(_) => Ok(2),
        KObject::CharList(s) => Ok(6 + s.len()),
        KObject::BooleanList(v) => Ok(6 + v.len()),
        KObject::GuidList(v) => Ok(6 + v.len() * 16),
        KObject::IntList(v) => Ok(6 + v.len() * 4),
        KObject::LongList(v) => Ok(6 + v.len() * 8),
        KObject::FloatList(v) => Ok(6 + v.len() * 8),
        KObject::DateList(v) => Ok(6 + v.len() * 4),
        KObject::DateTimeList(v) => Ok(6 + v.len() * 8),
        KObject::TimestampList(v) => Ok(6 + v.len() * 8),
        KObject::SymbolList(v) => Ok(6 + v.iter().map(|s| s.len() + 1).sum::<usize>()),
        KObject::List(v) => {
            let mut len = 6usize;
            for item in v {
                len += calc_len(item)?;
            }
            Ok(len)
        }
        KObject::Table {
            columns, values, ..
        } => {
            let mut len = 3usize; // kType(98) + attr + dict(99)
            // keys = symbol list
            len += calc_len(&KObject::SymbolList(columns.clone()))?;
            // values = general list header + each column
            len += 6;
            for v in values {
                len += 1 + calc_array_len(v)?;
            } // 1 for kType byte
            Ok(len)
        }
        KObject::Dict { keys, values } => Ok(1 + calc_len(keys)? + calc_len(values)?),
        _ => Err(KError::UnsupportedKType(0)),
    }
}

fn calc_array_len(obj: &KObject) -> Result<usize, KError> {
    match obj {
        KObject::BooleanList(v) => Ok(5 + v.len()),
        KObject::GuidList(v) => Ok(5 + v.len() * 16),
        KObject::IntList(v) => Ok(5 + v.len() * 4),
        KObject::LongList(v) => Ok(5 + v.len() * 8),
        KObject::FloatList(v) => Ok(5 + v.len() * 8),
        KObject::SymbolList(v) => Ok(5 + v.iter().map(|s| s.len() + 1).sum::<usize>()),
        KObject::DateList(v) => Ok(5 + v.len() * 4),
        KObject::DateTimeList(v) => Ok(5 + v.len() * 8),
        KObject::TimestampList(v) => Ok(5 + v.len() * 8),
        KObject::CharList(s) => Ok(5 + s.len()),
        KObject::List(v) => {
            let mut l = 5usize;
            for i in v {
                l += calc_len(i)?;
            }
            Ok(l)
        }
        _ => Err(KError::UnsupportedKType(0)),
    }
}

pub fn serialize(obj: &KObject) -> Result<Vec<u8>, KError> {
    let msg_len = calc_len(obj)?;
    let total = 8 + msg_len;
    let mut buf = vec![0u8; total];
    buf[0] = 1; // little-endian
    buf[4..8].copy_from_slice(&(total as u32).to_le_bytes());
    let mut pos = 8;
    write(&mut buf, &mut pos, obj)?;
    Ok(buf)
}

fn write(buf: &mut [u8], pos: &mut usize, obj: &KObject) -> Result<(), KError> {
    match obj {
        KObject::Null => {
            buf[*pos] = 101;
            *pos += 1;
            buf[*pos] = 0;
            *pos += 1;
        }
        KObject::Boolean(v) => {
            buf[*pos] = 255;
            *pos += 1;
            buf[*pos] = if *v { 1 } else { 0 };
            *pos += 1;
        }
        KObject::Float(v) => {
            buf[*pos] = 247;
            *pos += 1;
            buf[*pos..*pos + 8].copy_from_slice(&v.to_le_bytes());
            *pos += 8;
        }
        KObject::Long(v) => {
            buf[*pos] = 249;
            *pos += 1;
            buf[*pos..*pos + 8].copy_from_slice(&v.to_le_bytes());
            *pos += 8;
        }
        KObject::Timestamp(ns) => {
            buf[*pos] = 244;
            *pos += 1;
            buf[*pos..*pos + 8].copy_from_slice(&ns.to_le_bytes());
            *pos += 8;
        }
        KObject::CharList(s) => {
            buf[*pos] = 10;
            *pos += 1;
            write_array_header(buf, pos, s.len());
            buf[*pos..*pos + s.len()].copy_from_slice(s.as_bytes());
            *pos += s.len();
        }
        KObject::BooleanList(v) => {
            buf[*pos] = 1;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for b in v {
                buf[*pos] = if *b { 1 } else { 0 };
                *pos += 1;
            }
        }
        KObject::GuidList(v) => {
            buf[*pos] = 2;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for g in v {
                buf[*pos..*pos + 16].copy_from_slice(g);
                *pos += 16;
            }
        }
        KObject::IntList(v) => {
            buf[*pos] = 6;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for i in v {
                buf[*pos..*pos + 4].copy_from_slice(&i.to_le_bytes());
                *pos += 4;
            }
        }
        KObject::LongList(v) => {
            buf[*pos] = 7;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for l in v {
                buf[*pos..*pos + 8].copy_from_slice(&l.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::FloatList(v) => {
            buf[*pos] = 9;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for f in v {
                buf[*pos..*pos + 8].copy_from_slice(&f.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::SymbolList(v) => {
            buf[*pos] = 11;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for s in v {
                buf[*pos..*pos + s.len()].copy_from_slice(s.as_bytes());
                *pos += s.len();
                buf[*pos] = 0;
                *pos += 1;
            }
        }
        KObject::DateList(v) => {
            buf[*pos] = 14;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for d in v {
                buf[*pos..*pos + 4].copy_from_slice(&d.to_le_bytes());
                *pos += 4;
            }
        }
        KObject::DateTimeList(v) => {
            buf[*pos] = 15;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for d in v {
                buf[*pos..*pos + 8].copy_from_slice(&d.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::TimestampList(v) => {
            buf[*pos] = 12;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for t in v {
                buf[*pos..*pos + 8].copy_from_slice(&t.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::List(v) => {
            buf[*pos] = 0;
            *pos += 1;
            write_array_header(buf, pos, v.len());
            for item in v {
                write(buf, pos, item)?;
            }
        }
        KObject::Table {
            columns,
            values,
            col_types,
        } => {
            buf[*pos] = 98;
            *pos += 1;
            buf[*pos] = 0;
            *pos += 1; // attr
            buf[*pos] = 99;
            *pos += 1; // dict
            // Write column names
            write(buf, pos, &KObject::SymbolList(columns.clone()))?;
            // Write values as general list
            buf[*pos] = 0;
            *pos += 1; // kType 0 (general list)
            write_array_header(buf, pos, columns.len());
            for (i, v) in values.iter().enumerate() {
                // Write the kType byte for this column
                let ct = col_types.get(i).copied().unwrap_or(b' ');
                let k_idx = K_TYPE_CHAR.iter().position(|&c| c == ct).unwrap_or(0) as u8;
                buf[*pos] = k_idx;
                *pos += 1;
                write_array_body(buf, pos, v)?;
            }
        }
        KObject::Dict { keys, values } => {
            buf[*pos] = 99;
            *pos += 1;
            write(buf, pos, keys)?;
            write(buf, pos, values)?;
        }
        _ => return Err(KError::UnsupportedKType(0)),
    }
    Ok(())
}

fn write_array_header(buf: &mut [u8], pos: &mut usize, len: usize) {
    buf[*pos] = 0;
    *pos += 1; // attribute
    buf[*pos..*pos + 4].copy_from_slice(&(len as u32).to_le_bytes());
    *pos += 4;
}

fn write_array_body(buf: &mut [u8], pos: &mut usize, obj: &KObject) -> Result<(), KError> {
    match obj {
        KObject::BooleanList(v) => {
            write_array_header(buf, pos, v.len());
            for b in v {
                buf[*pos] = if *b { 1 } else { 0 };
                *pos += 1;
            }
        }
        KObject::GuidList(v) => {
            write_array_header(buf, pos, v.len());
            for g in v {
                buf[*pos..*pos + 16].copy_from_slice(g);
                *pos += 16;
            }
        }
        KObject::IntList(v) => {
            write_array_header(buf, pos, v.len());
            for i in v {
                buf[*pos..*pos + 4].copy_from_slice(&i.to_le_bytes());
                *pos += 4;
            }
        }
        KObject::LongList(v) => {
            write_array_header(buf, pos, v.len());
            for l in v {
                buf[*pos..*pos + 8].copy_from_slice(&l.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::FloatList(v) => {
            write_array_header(buf, pos, v.len());
            for f in v {
                buf[*pos..*pos + 8].copy_from_slice(&f.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::SymbolList(v) => {
            write_array_header(buf, pos, v.len());
            for s in v {
                buf[*pos..*pos + s.len()].copy_from_slice(s.as_bytes());
                *pos += s.len();
                buf[*pos] = 0;
                *pos += 1;
            }
        }
        KObject::DateList(v) => {
            write_array_header(buf, pos, v.len());
            for d in v {
                buf[*pos..*pos + 4].copy_from_slice(&d.to_le_bytes());
                *pos += 4;
            }
        }
        KObject::DateTimeList(v) => {
            write_array_header(buf, pos, v.len());
            for d in v {
                buf[*pos..*pos + 8].copy_from_slice(&d.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::TimestampList(v) => {
            write_array_header(buf, pos, v.len());
            for t in v {
                buf[*pos..*pos + 8].copy_from_slice(&t.to_le_bytes());
                *pos += 8;
            }
        }
        KObject::List(v) => {
            write_array_header(buf, pos, v.len());
            for item in v {
                write(buf, pos, item)?;
            }
        }
        KObject::CharList(s) => {
            write_array_header(buf, pos, s.len());
            buf[*pos..*pos + s.len()].copy_from_slice(s.as_bytes());
            *pos += s.len();
        }
        _ => return Err(KError::UnsupportedKType(0)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::deserialize;

    fn roundtrip(hex: &str) {
        let buf = hex::decode(hex).unwrap();
        let obj = deserialize(&buf).unwrap();
        let out = serialize(&obj).unwrap();
        assert_eq!(hex::encode(&out), hex, "roundtrip failed for {hex}");
    }

    #[test]
    fn test_null() {
        roundtrip("010000000a0000006500");
    }
    #[test]
    fn test_bool_true() {
        roundtrip("010000000a000000ff01");
    }
    #[test]
    fn test_bool_false() {
        roundtrip("010000000a000000ff00");
    }
    #[test]
    fn test_bool_list() {
        roundtrip("01000000100000000100020000000100");
    }
    #[test]
    fn test_string() {
        roundtrip("01000000120000000a00040000002e7a2e64");
    }
    #[test]
    fn test_symbols() {
        roundtrip("01000000120000000b000200000061006200");
    }
    #[test]
    fn test_float() {
        roundtrip("0100000011000000f70000000000c05840");
    }
    #[test]
    fn test_bigint() {
        roundtrip("0100000011000000f90100000000000000");
    }

    #[test]
    fn test_int_list() {
        roundtrip("010000001e0000000600040000006300000000000080ffffff7f01000080");
    }

    #[test]
    fn test_long_list() {
        roundtrip(
            "010000002e00000007000400000063000000000000000000000000000080ffffffffffffff7f0100000000000080",
        );
    }

    #[test]
    fn test_float_list() {
        // Use the roundtrip hex from JS test (line 207-208)
        roundtrip(
            "010000002e000000090004000000\
0000000000c05840000000000000f87f000000000000f07f000000000000f0ff",
        );
    }

    #[test]
    fn test_dict() {
        // From JS test line 479-481
        roundtrip(
            "0100000034000000\
630b000200000073796d00707269636500\
0000020000000a0006000000383330362e54f79a99999999e18440",
        );
    }

    #[test]
    fn test_table() {
        roundtrip(
            "01000000590000006200630b000300000073796d0064617465006f70656e000000030000000b000200000041584a4f0041584a4f000e00020000007b1e00007c1e0000090002000000000000000058bb40000000000070b740",
        );
    }
}
