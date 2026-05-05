/// Rust-native representation of kdb+ values.
///
/// Each variant corresponds to a kdb+ type or compound structure.
/// Typed vectors avoid boxing overhead for homogeneous lists.
#[derive(Debug, Clone, PartialEq)]
pub enum KObject {
    // --- Atoms ---
    Null,
    Boolean(bool),
    Guid([u8; 16]),
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    Real(f32),
    Float(f64),
    Char(char),
    Symbol(String),
    Timestamp(i64), // nanoseconds since kdb+ epoch (2000-01-01)
    Month(i32),
    Date(i32),     // days since kdb+ epoch
    DateTime(f64), // fractional days since kdb+ epoch
    Timespan(i64), // nanoseconds
    Minute(i32),
    Second(i32),
    Time(i32), // milliseconds

    // --- Lists ---
    /// Mixed/general list (kType 0)
    List(Vec<KObject>),
    BooleanList(Vec<bool>),
    GuidList(Vec<[u8; 16]>),
    ByteList(Vec<u8>),
    ShortList(Vec<i16>),
    IntList(Vec<i32>),
    LongList(Vec<i64>),
    RealList(Vec<f32>),
    FloatList(Vec<f64>),
    /// Char list = string (kType 10)
    CharList(String),
    SymbolList(Vec<String>),
    TimestampList(Vec<i64>),
    MonthList(Vec<i32>),
    DateList(Vec<i32>),
    DateTimeList(Vec<f64>),
    TimespanList(Vec<i64>),
    MinuteList(Vec<i32>),
    SecondList(Vec<i32>),
    TimeList(Vec<i32>),

    // --- Compound structures ---
    Dict {
        keys: Box<KObject>,
        values: Box<KObject>,
    },
    Table {
        columns: Vec<String>,
        /// One KObject per column (typed list)
        values: Vec<KObject>,
        /// kType char for each column (e.g. 's', 'd', 'f')
        col_types: Vec<u8>,
    },
    KeyedTable {
        /// Key columns
        key_columns: Vec<String>,
        /// All columns (keys + values)
        columns: Vec<String>,
        values: Vec<KObject>,
        col_types: Vec<u8>,
    },
    /// Flip table: dict 99 where key is a table — deserialized as list of row dicts
    FlipRows(Vec<KObject>),

    // --- Functions / operators ---
    UnaryPrimitive(u8),
    Operator(u8),
    Iterator(u8),
    Lambda(String),
    Projection(Vec<KObject>),

    // --- Special ---
    /// kdb+ error (kType 128)
    Error(String),
}

impl KObject {
    /// Returns the kType character for typed lists, used for Symbol.for('kType').
    pub fn k_type_char(&self) -> Option<u8> {
        match self {
            KObject::BooleanList(_) => Some(b'b'),
            KObject::GuidList(_) => Some(b'g'),
            KObject::ByteList(_) => Some(b'x'),
            KObject::ShortList(_) => Some(b'h'),
            KObject::IntList(_) => Some(b'i'),
            KObject::LongList(_) => Some(b'j'),
            KObject::RealList(_) => Some(b'e'),
            KObject::FloatList(_) => Some(b'f'),
            KObject::CharList(_) => Some(b'c'),
            KObject::SymbolList(_) => Some(b's'),
            KObject::TimestampList(_) => Some(b'p'),
            KObject::MonthList(_) => Some(b'm'),
            KObject::DateList(_) => Some(b'd'),
            KObject::DateTimeList(_) => Some(b'z'),
            KObject::TimespanList(_) => Some(b'n'),
            KObject::MinuteList(_) => Some(b'u'),
            KObject::SecondList(_) => Some(b'v'),
            KObject::TimeList(_) => Some(b't'),
            KObject::List(_) => Some(b' '),
            _ => None,
        }
    }

    /// Returns the number of elements for list variants, 0 for atoms.
    pub fn len(&self) -> usize {
        match self {
            KObject::BooleanList(v) => v.len(),
            KObject::GuidList(v) => v.len(),
            KObject::ByteList(v) => v.len(),
            KObject::ShortList(v) => v.len(),
            KObject::IntList(v) => v.len(),
            KObject::LongList(v) => v.len(),
            KObject::RealList(v) => v.len(),
            KObject::FloatList(v) => v.len(),
            KObject::CharList(s) => s.len(),
            KObject::SymbolList(v) => v.len(),
            KObject::TimestampList(v) => v.len(),
            KObject::MonthList(v) => v.len(),
            KObject::DateList(v) => v.len(),
            KObject::DateTimeList(v) => v.len(),
            KObject::TimespanList(v) => v.len(),
            KObject::MinuteList(v) => v.len(),
            KObject::SecondList(v) => v.len(),
            KObject::TimeList(v) => v.len(),
            KObject::List(v) => v.len(),
            KObject::Projection(v) => v.len(),
            KObject::FlipRows(v) => v.len(),
            _ => 0,
        }
    }
}
