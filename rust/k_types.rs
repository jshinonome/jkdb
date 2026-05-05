/// KDB+ IPC type constants, sentinel values, size tables, and operator lookup maps.

// --- Null sentinels ---
pub const SHORT_NULL: i16 = i16::MIN; // -32768
pub const SHORT_POS_INF: i16 = i16::MAX; // 32767
pub const SHORT_NEG_INF: i16 = i16::MAX - 1; // -32767 wraps; use constant
pub const INT_NULL: i32 = i32::MIN;
pub const INT_POS_INF: i32 = i32::MAX;
pub const INT_NEG_INF: i32 = i32::MIN + 1;
pub const LONG_NULL: i64 = i64::MIN;
pub const LONG_POS_INF: i64 = i64::MAX;
pub const LONG_NEG_INF: i64 = i64::MIN + 1;

// --- Temporal constants ---
/// Milliseconds between Unix epoch (1970-01-01) and kdb+ epoch (2000-01-01)
pub const MS_DIFF: i64 = 946_684_800_000;
pub const MS_PER_DAY: i64 = 86_400_000;

// --- kType character mapping: index = kType, value = char ---
pub const K_TYPE_CHAR: &[u8] = b" bg xhijefcspmdznuvt";

/// Byte size of each kType atom.
pub fn size_by_k_type(k_type: u8) -> Option<usize> {
    match k_type {
        1 => Some(1),
        2 => Some(16),
        4 => Some(1),
        5 => Some(2),
        6 => Some(4),
        7 => Some(8),
        8 => Some(4),
        9 => Some(8),
        10 => Some(1),
        12 => Some(8),
        13 => Some(4),
        14 => Some(4),
        15 => Some(8),
        16 => Some(8),
        17 => Some(4),
        18 => Some(4),
        19 => Some(4),
        101 => Some(1),
        _ => None,
    }
}

/// K101: unary primitive lookup
pub fn k101_name(code: u8) -> Option<&'static str> {
    match code {
        0 => None, // null
        1 => Some("+:"),
        2 => Some("-:"),
        3 => Some("*:"),
        4 => Some("%:"),
        5 => Some("&:"),
        6 => Some("|:"),
        7 => Some("^:"),
        8 => Some("=:"),
        9 => Some("<:"),
        10 => Some(">:"),
        11 => Some("$:"),
        12 => Some(",:"),
        13 => Some("#:"),
        14 => Some("_:"),
        15 => Some("~:"),
        16 => Some("!:"),
        17 => Some("?:"),
        18 => Some("@:"),
        19 => Some(".:"),
        20 => Some("0::"),
        21 => Some("1::"),
        22 => Some("2::"),
        23 => Some("avg"),
        24 => Some("last"),
        25 => Some("sum"),
        26 => Some("prd"),
        27 => Some("min"),
        28 => Some("max"),
        29 => Some("exit"),
        30 => Some("getenv"),
        31 => Some("abs"),
        32 => Some("sqrt"),
        33 => Some("log"),
        34 => Some("exp"),
        35 => Some("sin"),
        36 => Some("asin"),
        37 => Some("cos"),
        38 => Some("acos"),
        39 => Some("tan"),
        40 => Some("atan"),
        41 => Some("enlist"),
        42 => Some("var"),
        43 => Some("dev"),
        44 => Some("hopen"),
        255 => Some("::"),
        _ => None,
    }
}

/// K102: binary operator lookup
pub fn k102_name(code: u8) -> Option<&'static str> {
    match code {
        0 => Some(":"),
        1 => Some("+"),
        2 => Some("-"),
        3 => Some("*"),
        4 => Some("%"),
        5 => Some("&"),
        6 => Some("|"),
        7 => Some("^"),
        8 => Some("="),
        9 => Some("<"),
        10 => Some(">"),
        11 => Some("$"),
        12 => Some(","),
        13 => Some("#"),
        14 => Some("_"),
        15 => Some("~"),
        16 => Some("!"),
        17 => Some("?"),
        18 => Some("@"),
        19 => Some("."),
        20 => Some("0:"),
        21 => Some("1:"),
        22 => Some("2:"),
        23 => Some("in"),
        24 => Some("within"),
        25 => Some("like"),
        26 => Some("bin"),
        27 => Some("ss"),
        28 => Some("insert"),
        29 => Some("wsum"),
        30 => Some("wavg"),
        31 => Some("div"),
        32 => Some("xexp"),
        33 => Some("setenv"),
        34 => Some("binr"),
        35 => Some("cov"),
        36 => Some("cor"),
        _ => None,
    }
}

/// K103: iterator lookup
pub fn k103_name(code: u8) -> Option<&'static str> {
    match code {
        0 => Some("'"),
        1 => Some("/"),
        2 => Some("\\"),
        3 => Some("':"),
        4 => Some("/:"),
        5 => Some("\\:"),
        _ => None,
    }
}

// Short null/infinity sentinel check
pub const SHORT_NEG_INF_VAL: i16 = -32767;
