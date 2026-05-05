use js_sys::{Array, BigInt as JsBigInt, Date as JsDate, Object, Reflect, Symbol};
/// WASM bindings via wasm-bindgen.
///
/// Exposes `ipc_deserialize` and `ipc_serialize` to JavaScript.
use wasm_bindgen::prelude::*;

use crate::deserialize::{DeserializeOptions, deserialize};
use crate::k_types::*;
use crate::kobject::KObject;
use crate::serialize;
use crate::temporal::{bigint_to_timespan, int_to_temporal};

#[wasm_bindgen]
pub fn ipc_deserialize(
    buffer: &[u8],
    use_big_int: bool,
    include_nanosecond: bool,
    date_to_millisecond: bool,
) -> Result<JsValue, JsError> {
    let opts = DeserializeOptions {
        use_big_int,
        include_nanosecond,
        date_to_millisecond,
    };
    let obj = deserialize(buffer).map_err(|e| JsError::new(&e.to_string()))?;
    kobject_to_js(&obj, &opts).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn ipc_serialize(val: JsValue) -> Result<Vec<u8>, JsError> {
    let obj = js_to_kobject(&val).map_err(|e| JsError::new(&e.to_string()))?;
    serialize::serialize(&obj).map_err(|e| JsError::new(&e.to_string()))
}

/// Convert KObject to JsValue, applying formatting options.
pub fn kobject_to_js(obj: &KObject, opts: &DeserializeOptions) -> Result<JsValue, String> {
    match obj {
        KObject::Null => Ok(JsValue::NULL),
        KObject::Boolean(b) => Ok(JsValue::from(*b)),
        KObject::Byte(v) => Ok(JsValue::from(*v)),
        KObject::Short(v) => Ok(match *v {
            SHORT_NULL => JsValue::from(f64::NAN),
            SHORT_POS_INF => JsValue::from(f64::INFINITY),
            v if v == SHORT_NEG_INF_VAL => JsValue::from(f64::NEG_INFINITY),
            _ => JsValue::from(*v),
        }),
        KObject::Int(v) => Ok(match *v {
            INT_NULL => JsValue::from(f64::NAN),
            INT_POS_INF => JsValue::from(f64::INFINITY),
            INT_NEG_INF => JsValue::from(f64::NEG_INFINITY),
            _ => JsValue::from(*v),
        }),
        KObject::Long(v) => Ok(match *v {
            LONG_NULL => JsValue::from(f64::NAN),
            LONG_POS_INF => JsValue::from(f64::INFINITY),
            LONG_NEG_INF => JsValue::from(f64::NEG_INFINITY),
            _ if opts.use_big_int => JsBigInt::from(*v).into(),
            _ => JsValue::from(*v as f64),
        }),
        KObject::Real(v) => Ok(JsValue::from(*v as f64)),
        KObject::Float(v) => Ok(JsValue::from(*v)),
        KObject::Char(c) => Ok(JsValue::from(c.to_string())),
        KObject::Symbol(s) => Ok(JsValue::from(s.as_str())),
        KObject::Guid(g) => Ok(JsValue::from(hex::encode(g))),
        KObject::CharList(s) => Ok(JsValue::from(s.as_str())),

        KObject::Timestamp(ns) => {
            if *ns == LONG_NULL || *ns == LONG_POS_INF || *ns == LONG_NEG_INF {
                return Ok(if opts.include_nanosecond {
                    JsValue::from("")
                } else {
                    JsValue::NULL
                });
            }
            let ms = (*ns / 1_000_000) + MS_DIFF;
            if opts.include_nanosecond {
                let date = JsDate::new(&JsValue::from(ms as f64));
                let iso = date.to_iso_string();
                let iso_str: String = iso.into();
                let nano = (*ns % 1_000_000).unsigned_abs();
                Ok(JsValue::from(format!(
                    "{}{:06}",
                    &iso_str[..iso_str.len() - 1],
                    nano
                )))
            } else {
                Ok(JsDate::new(&JsValue::from(ms as f64)).into())
            }
        }
        KObject::Month(v) => {
            Ok(int_to_temporal(*v, 13).map_or(JsValue::NULL, |s| JsValue::from(s)))
        }
        KObject::Date(v) => {
            if *v == INT_NULL || *v == INT_POS_INF || *v == INT_NEG_INF {
                return Ok(if opts.date_to_millisecond {
                    match *v {
                        INT_NULL => JsValue::from(f64::NAN),
                        INT_POS_INF => JsValue::from(f64::INFINITY),
                        _ => JsValue::from(f64::NEG_INFINITY),
                    }
                } else {
                    JsValue::NULL
                });
            }
            let ms = MS_DIFF + (*v as i64) * MS_PER_DAY;
            if opts.date_to_millisecond {
                Ok(JsValue::from(ms as f64))
            } else {
                Ok(JsDate::new(&JsValue::from(ms as f64)).into())
            }
        }
        KObject::DateTime(v) => {
            if v.is_nan() || v.is_infinite() {
                return Ok(if opts.date_to_millisecond {
                    JsValue::from(*v * MS_PER_DAY as f64 + MS_DIFF as f64)
                } else {
                    JsValue::NULL
                });
            }
            let ms = MS_DIFF as f64 + *v * MS_PER_DAY as f64;
            if opts.date_to_millisecond {
                Ok(JsValue::from(ms))
            } else {
                Ok(JsDate::new(&JsValue::from(ms)).into())
            }
        }
        KObject::Timespan(ns) => {
            Ok(bigint_to_timespan(*ns).map_or(JsValue::NULL, |s| JsValue::from(s)))
        }
        KObject::Minute(v) => {
            Ok(int_to_temporal(*v, 17).map_or(JsValue::NULL, |s| JsValue::from(s)))
        }
        KObject::Second(v) => {
            Ok(int_to_temporal(*v, 18).map_or(JsValue::NULL, |s| JsValue::from(s)))
        }
        KObject::Time(v) => Ok(int_to_temporal(*v, 19).map_or(JsValue::NULL, |s| JsValue::from(s))),

        KObject::UnaryPrimitive(c) => Ok(k101_name(*c).map_or(JsValue::NULL, |s| JsValue::from(s))),
        KObject::Operator(c) => Ok(k102_name(*c).map_or(JsValue::NULL, |s| JsValue::from(s))),
        KObject::Iterator(c) => Ok(k103_name(*c).map_or(JsValue::NULL, |s| JsValue::from(s))),
        KObject::Lambda(s) | KObject::Error(s) => Ok(JsValue::from(s.as_str())),

        // Lists
        KObject::List(v) => {
            let arr = Array::new_with_length(v.len() as u32);
            for (i, item) in v.iter().enumerate() {
                arr.set(i as u32, kobject_to_js(item, opts)?);
            }
            set_k_type(&arr, b' ');
            Ok(arr.into())
        }
        KObject::BooleanList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, b) in v.iter().enumerate() {
                a.set(i as u32, JsValue::from(*b));
            }
            set_k_type(&a, b'b');
            Ok(a.into())
        }
        KObject::GuidList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, g) in v.iter().enumerate() {
                a.set(i as u32, JsValue::from(hex::encode(g)));
            }
            set_k_type(&a, b'g');
            Ok(a.into())
        }
        KObject::ByteList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, b) in v.iter().enumerate() {
                a.set(i as u32, JsValue::from(*b));
            }
            set_k_type(&a, b'x');
            Ok(a.into())
        }
        KObject::ShortList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, s) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Short(*s), opts)?);
            }
            set_k_type(&a, b'h');
            Ok(a.into())
        }
        KObject::IntList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, n) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Int(*n), opts)?);
            }
            set_k_type(&a, b'i');
            Ok(a.into())
        }
        KObject::LongList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, n) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Long(*n), opts)?);
            }
            set_k_type(&a, b'j');
            Ok(a.into())
        }
        KObject::RealList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, f) in v.iter().enumerate() {
                a.set(i as u32, JsValue::from(*f as f64));
            }
            set_k_type(&a, b'e');
            Ok(a.into())
        }
        KObject::FloatList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, f) in v.iter().enumerate() {
                a.set(i as u32, JsValue::from(*f));
            }
            set_k_type(&a, b'f');
            Ok(a.into())
        }
        KObject::SymbolList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, s) in v.iter().enumerate() {
                a.set(i as u32, JsValue::from(s.as_str()));
            }
            set_k_type(&a, b's');
            Ok(a.into())
        }
        KObject::TimestampList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, ns) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Timestamp(*ns), opts)?);
            }
            set_k_type(&a, b'p');
            Ok(a.into())
        }
        KObject::MonthList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, m) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Month(*m), opts)?);
            }
            set_k_type(&a, b'm');
            Ok(a.into())
        }
        KObject::DateList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, d) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Date(*d), opts)?);
            }
            set_k_type(&a, b'd');
            Ok(a.into())
        }
        KObject::DateTimeList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, d) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::DateTime(*d), opts)?);
            }
            set_k_type(&a, b'z');
            Ok(a.into())
        }
        KObject::TimespanList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, ns) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Timespan(*ns), opts)?);
            }
            set_k_type(&a, b'n');
            Ok(a.into())
        }
        KObject::MinuteList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, m) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Minute(*m), opts)?);
            }
            set_k_type(&a, b'u');
            Ok(a.into())
        }
        KObject::SecondList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, s) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Second(*s), opts)?);
            }
            set_k_type(&a, b'v');
            Ok(a.into())
        }
        KObject::TimeList(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, t) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(&KObject::Time(*t), opts)?);
            }
            set_k_type(&a, b't');
            Ok(a.into())
        }
        KObject::Projection(v) => {
            let a = Array::new_with_length(v.len() as u32);
            for (i, item) in v.iter().enumerate() {
                a.set(i as u32, kobject_to_js(item, opts)?);
            }
            Ok(a.into())
        }

        // Dict → JS Object
        KObject::Dict { keys, values } => {
            if let KObject::SymbolList(ks) = keys.as_ref() {
                let obj = Object::new();
                let vals = match values.as_ref() {
                    KObject::List(v) => v
                        .iter()
                        .map(|item| kobject_to_js(item, opts))
                        .collect::<Result<Vec<_>, _>>()?,
                    KObject::FlipRows(rows) => rows
                        .iter()
                        .map(|r| kobject_to_js(r, opts))
                        .collect::<Result<Vec<_>, _>>()?,
                    // Typed lists: extract each element and convert individually
                    typed_list if typed_list.len() > 0 => (0..typed_list.len())
                        .map(|i| {
                            let elem = crate::deserialize::get_element(typed_list, i);
                            kobject_to_js(&elem, opts)
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => vec![kobject_to_js(values, opts)?],
                };
                for (i, k) in ks.iter().enumerate() {
                    if let Some(v) = vals.get(i) {
                        Reflect::set(&obj, &JsValue::from(k.as_str()), v).ok();
                    }
                }
                Ok(obj.into())
            } else {
                Ok(JsValue::NULL)
            }
        }

        // Table → JS Object with Symbol.for('meta')
        KObject::Table {
            columns,
            values,
            col_types,
        } => {
            let obj = Object::new();
            for (i, col) in columns.iter().enumerate() {
                Reflect::set(
                    &obj,
                    &JsValue::from(col.as_str()),
                    &kobject_to_js(&values[i], opts)?,
                )
                .ok();
            }
            // Set meta
            let meta = Object::new();
            let c_arr = Array::new_with_length(columns.len() as u32);
            let t_arr = Array::new_with_length(col_types.len() as u32);
            for (i, c) in columns.iter().enumerate() {
                c_arr.set(i as u32, JsValue::from(c.as_str()));
            }
            for (i, t) in col_types.iter().enumerate() {
                t_arr.set(i as u32, JsValue::from(String::from(*t as char)));
            }
            Reflect::set(&meta, &JsValue::from("c"), &c_arr).ok();
            Reflect::set(&meta, &JsValue::from("t"), &t_arr).ok();
            let meta_sym = Symbol::for_("meta");
            Reflect::set(&obj, &meta_sym, &meta).ok();
            Ok(obj.into())
        }

        KObject::KeyedTable {
            key_columns,
            columns,
            values,
            col_types,
        } => {
            let obj = Object::new();
            for (i, col) in columns.iter().enumerate() {
                Reflect::set(
                    &obj,
                    &JsValue::from(col.as_str()),
                    &kobject_to_js(&values[i], opts)?,
                )
                .ok();
            }
            let meta = Object::new();
            let c_arr = Array::new_with_length(columns.len() as u32);
            let t_arr = Array::new_with_length(col_types.len() as u32);
            for (i, c) in columns.iter().enumerate() {
                c_arr.set(i as u32, JsValue::from(c.as_str()));
            }
            for (i, t) in col_types.iter().enumerate() {
                t_arr.set(i as u32, JsValue::from(String::from(*t as char)));
            }
            Reflect::set(&meta, &JsValue::from("c"), &c_arr).ok();
            Reflect::set(&meta, &JsValue::from("t"), &t_arr).ok();
            Reflect::set(&obj, &Symbol::for_("meta"), &meta).ok();
            // Set keys
            let keys_arr = Array::new_with_length(key_columns.len() as u32);
            for (i, k) in key_columns.iter().enumerate() {
                keys_arr.set(i as u32, JsValue::from(k.as_str()));
            }
            Reflect::set(&obj, &Symbol::for_("keys"), &keys_arr).ok();
            Ok(obj.into())
        }

        KObject::FlipRows(rows) => {
            let arr = Array::new_with_length(rows.len() as u32);
            for (i, r) in rows.iter().enumerate() {
                arr.set(i as u32, kobject_to_js(r, opts)?);
            }
            Ok(arr.into())
        }
    }
}

fn set_k_type(arr: &Array, ch: u8) {
    let sym = Symbol::for_("kType");
    Reflect::set(arr, &sym, &JsValue::from(String::from(ch as char))).ok();
}

/// Convert a JS BigInt to i64 via string conversion.
fn bigint_to_i64(val: &JsValue) -> Result<i64, String> {
    let bi = JsBigInt::new(val).map_err(|_| "Not a BigInt")?;
    let s = bi.to_string(10).map_err(|_| "BigInt toString failed")?;
    let rust_str: String = s.into();
    rust_str.parse::<i64>().map_err(|e| format!("BigInt parse error: {e}"))
}

/// Convert JsValue → KObject (for serialization).
pub fn js_to_kobject(val: &JsValue) -> Result<KObject, String> {
    if val.is_null() || val.is_undefined() {
        return Ok(KObject::Null);
    }
    if let Some(b) = val.as_bool() {
        return Ok(KObject::Boolean(b));
    }
    if let Some(f) = val.as_f64() {
        return Ok(KObject::Float(f));
    }
    if let Some(s) = val.as_string() {
        return Ok(KObject::CharList(s));
    }

    // BigInt → Long
    if val.is_bigint() {
        let n = bigint_to_i64(val)?;
        return Ok(KObject::Long(n));
    }

    // Date → Timestamp (nanoseconds since kdb+ epoch)
    if val.is_instance_of::<JsDate>() {
        let date = JsDate::from(val.clone());
        let ms = date.get_time();
        if ms.is_nan() {
            return Ok(KObject::Timestamp(LONG_NULL));
        }
        let ns = ((ms as i64) - MS_DIFF) * 1_000_000;
        return Ok(KObject::Timestamp(ns));
    }

    // Array → typed list or general list
    if Array::is_array(val) {
        let arr = Array::from(val);
        let len = arr.length() as usize;
        let k_type_sym = Symbol::for_("kType");
        let k_type_val = Reflect::get(val, &k_type_sym).ok();
        let k_type = k_type_val.as_ref().and_then(|v| v.as_string());

        return match k_type.as_deref() {
            Some("b") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    v.push(arr.get(i as u32).as_bool().unwrap_or(false));
                }
                Ok(KObject::BooleanList(v))
            }
            Some("g") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let s = arr.get(i as u32).as_string().unwrap_or_default();
                    let bytes = hex::decode(&s).map_err(|e| e.to_string())?;
                    let mut g = [0u8; 16];
                    if bytes.len() == 16 { g.copy_from_slice(&bytes); }
                    v.push(g);
                }
                Ok(KObject::GuidList(v))
            }
            Some("x") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    v.push(arr.get(i as u32).as_f64().unwrap_or(0.0) as u8);
                }
                Ok(KObject::ByteList(v))
            }
            Some("h") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let val = arr.get(i as u32);
                    v.push(f64_to_short(val.as_f64().unwrap_or(0.0)));
                }
                Ok(KObject::ShortList(v))
            }
            Some("i") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let val = arr.get(i as u32);
                    v.push(f64_to_int(val.as_f64().unwrap_or(0.0)));
                }
                Ok(KObject::IntList(v))
            }
            Some("j") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let item = arr.get(i as u32);
                    if item.is_bigint() {
                        v.push(bigint_to_i64(&item)?);
                    } else {
                        v.push(f64_to_long(item.as_f64().unwrap_or(0.0)));
                    }
                }
                Ok(KObject::LongList(v))
            }
            Some("e") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let val = arr.get(i as u32);
                    v.push(f64_to_real(val.as_f64().unwrap_or(0.0)));
                }
                Ok(KObject::RealList(v))
            }
            Some("f") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let val = arr.get(i as u32);
                    v.push(f64_to_float(val.as_f64().unwrap_or(0.0)));
                }
                Ok(KObject::FloatList(v))
            }
            Some("s") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    v.push(arr.get(i as u32).as_string().unwrap_or_default());
                }
                Ok(KObject::SymbolList(v))
            }
            Some("p") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let item = arr.get(i as u32);
                    v.push(js_to_timestamp_ns(&item));
                }
                Ok(KObject::TimestampList(v))
            }
            Some("d") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let item = arr.get(i as u32);
                    v.push(js_to_date_days(&item));
                }
                Ok(KObject::DateList(v))
            }
            Some("z") => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    let item = arr.get(i as u32);
                    v.push(js_to_datetime_frac(&item));
                }
                Ok(KObject::DateTimeList(v))
            }
            // General list (kType ' ' or no kType)
            _ => {
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    v.push(js_to_kobject(&arr.get(i as u32))?);
                }
                Ok(KObject::List(v))
            }
        };
    }

    // Object → Table (with meta) or Dict (without meta)
    if val.is_object() {
        let obj = Object::from(val.clone());
        let meta_sym = Symbol::for_("meta");
        let meta = Reflect::get(val, &meta_sym).ok();
        let keys = Object::keys(&obj);
        let key_count = keys.length() as usize;

        if let Some(meta_val) = meta.filter(|v| v.is_object() && !v.is_null()) {
            // Table: Object with Symbol.for('meta')
            let meta_c = Reflect::get(&meta_val, &JsValue::from("c"))
                .map_err(|_| "meta missing 'c'")?;
            let meta_t = Reflect::get(&meta_val, &JsValue::from("t"))
                .map_err(|_| "meta missing 't'")?;
            let c_arr = Array::from(&meta_c);
            let t_arr = Array::from(&meta_t);
            let ncols = c_arr.length() as usize;

            let mut columns = Vec::with_capacity(ncols);
            let mut values = Vec::with_capacity(ncols);
            let mut col_types = Vec::with_capacity(ncols);

            for i in 0..ncols {
                let col = c_arr.get(i as u32).as_string().unwrap_or_default();
                let typ = t_arr.get(i as u32).as_string().unwrap_or_default();
                let col_val = Reflect::get(val, &JsValue::from(&col))
                    .map_err(|_| format!("missing column {col}"))?;

                // Build typed list from column array
                let col_arr = Array::from(&col_val);
                let col_k_type_sym = Symbol::for_("kType");
                // Set kType on the column array for js_to_kobject to pick up
                Reflect::set(&col_arr, &col_k_type_sym, &JsValue::from(&typ)).ok();
                let col_obj = js_to_kobject(&col_arr.into())?;

                col_types.push(typ.as_bytes().first().copied().unwrap_or(b' '));
                columns.push(col);
                values.push(col_obj);
            }

            return Ok(KObject::Table { columns, values, col_types });
        } else {
            // Dict: Object without meta → SymbolList keys, List/typed values
            let mut ks = Vec::with_capacity(key_count);
            let mut vs = Vec::with_capacity(key_count);

            for i in 0..key_count {
                let key = keys.get(i as u32).as_string().unwrap_or_default();
                let val_item = Reflect::get(val, &JsValue::from(&key))
                    .map_err(|_| format!("missing key {key}"))?;
                ks.push(key);
                vs.push(js_to_kobject(&val_item)?);
            }

            return Ok(KObject::Dict {
                keys: Box::new(KObject::SymbolList(ks)),
                values: Box::new(KObject::List(vs)),
            });
        }
    }

    Err("Unsupported JS type for serialization".to_string())
}

/// Convert JS f64 to i16, mapping NaN/Inf to sentinels.
fn f64_to_short(f: f64) -> i16 {
    if f.is_nan() { SHORT_NULL }
    else if f == f64::INFINITY { SHORT_POS_INF }
    else if f == f64::NEG_INFINITY { SHORT_NEG_INF_VAL }
    else { f as i16 }
}

/// Convert JS f64 to i32, mapping NaN/Inf to sentinels.
fn f64_to_int(f: f64) -> i32 {
    if f.is_nan() { INT_NULL }
    else if f == f64::INFINITY { INT_POS_INF }
    else if f == f64::NEG_INFINITY { INT_NEG_INF }
    else { f as i32 }
}

/// Convert JS f64 to i64, mapping NaN/Inf to sentinels.
fn f64_to_long(f: f64) -> i64 {
    if f.is_nan() { LONG_NULL }
    else if f == f64::INFINITY { LONG_POS_INF }
    else if f == f64::NEG_INFINITY { LONG_NEG_INF }
    else { f as i64 }
}

/// Convert JS f64 to f32, mapping NaN to kdb+ NaN sentinel.
fn f64_to_real(f: f64) -> f32 {
    if f.is_nan() { f32::from_bits(0xffc00000) }
    else { f as f32 }
}

/// Convert JS f64 to f64, mapping NaN to kdb+ NaN sentinel (0x7ff8...).
fn f64_to_float(f: f64) -> f64 {
    if f.is_nan() { f64::from_bits(0x7ff8_0000_0000_0000) }
    else { f }
}

/// Convert a JS Date to kdb+ timestamp nanoseconds.
fn js_to_timestamp_ns(val: &JsValue) -> i64 {
    if val.is_null() || val.is_undefined() {
        return LONG_NULL;
    }
    if val.is_instance_of::<JsDate>() {
        let date = JsDate::from(val.clone());
        let ms = date.get_time();
        if ms.is_nan() { return LONG_NULL; }
        return ((ms as i64) - MS_DIFF) * 1_000_000;
    }
    LONG_NULL
}

/// Convert a JS Date to kdb+ date (days since 2000-01-01).
fn js_to_date_days(val: &JsValue) -> i32 {
    if val.is_null() || val.is_undefined() {
        return INT_NULL;
    }
    if val.is_instance_of::<JsDate>() {
        let date = JsDate::from(val.clone());
        let ms = date.get_time();
        if ms.is_nan() { return INT_NULL; }
        return ((ms as i64 - MS_DIFF) / MS_PER_DAY) as i32;
    }
    INT_NULL
}

/// Convert a JS Date to kdb+ datetime (fractional days since 2000-01-01).
fn js_to_datetime_frac(val: &JsValue) -> f64 {
    if val.is_null() || val.is_undefined() {
        return f64::from_bits(0x7ff8_0000_0000_0000); // NaN
    }
    if val.is_instance_of::<JsDate>() {
        let date = JsDate::from(val.clone());
        let ms = date.get_time();
        if ms.is_nan() { return f64::from_bits(0x7ff8_0000_0000_0000); }
        return (ms - MS_DIFF as f64) / MS_PER_DAY as f64;
    }
    f64::from_bits(0x7ff8_0000_0000_0000)
}
