/// Temporal type formatting helpers.
///
/// Converts kdb+ internal representations (ints, nanoseconds) into human-readable strings.
use crate::k_types::{INT_NEG_INF, INT_NULL, INT_POS_INF, LONG_NEG_INF, LONG_NULL, LONG_POS_INF};

/// Convert nanoseconds (i64) to timespan string, e.g. "0D05:51:50.218577000"
pub fn bigint_to_timespan(ns: i64) -> Option<String> {
    if ns == LONG_NULL || ns == LONG_POS_INF || ns == LONG_NEG_INF {
        return None;
    }

    let sign = if ns < 0 { "-" } else { "" };
    let abs_ns = ns.unsigned_abs();
    let second = abs_ns / 1_000_000_000;
    let days = abs_ns / 86_400_000_000_000;
    let hours = (second / 3600) % 24;
    let minutes = (second / 60) % 60;
    let secs = second % 60;
    let nano = abs_ns % 1_000_000_000;

    Some(format!(
        "{sign}{days}D{hours:02}:{minutes:02}:{secs:02}.{nano:09}"
    ))
}

/// Convert kdb+ int to temporal string based on kType.
///
/// kType 243/13 = month, 239/17 = minute, 238/18 = second, 237/19 = time (ms)
pub fn int_to_temporal(unit: i32, k_type: u8) -> Option<String> {
    if unit == INT_NULL || unit == INT_POS_INF || unit == INT_NEG_INF {
        return None;
    }

    let sign = if unit < 0 { "-" } else { "" };
    let abs_unit = unit.unsigned_abs();

    match k_type {
        // month
        243 | 13 => {
            // JS uses `>>> 0` for unsigned right shift; Rust integer division truncates toward zero.
            // For negative: -147/12 = -12 in JS (floor), but -12 in Rust (truncate toward zero).
            // JS: 2000 + (-147/12 >>> 0) where >>> 0 on negative gives floor division
            // Actually JS: (unit / 12) >>> 0 performs floor for positive, but for negative
            // the JS code uses: 2000 + (unit / 12) >> 0 which truncates.
            // Then unit % 12 can be negative, and mm + 13 adjusts.
            let yyyy = if unit >= 0 {
                2000 + unit / 12
            } else {
                2000 + (unit - 11) / 12
            };
            let mm = ((unit % 12) + 12) % 12; // always positive [0..11]
            Some(format!("{yyyy}.{:02}m", mm + 1))
        }
        // minute
        239 | 17 => {
            let hh = abs_unit / 60;
            let mm = abs_unit % 60;
            Some(format!("{sign}{hh:02}:{mm:02}"))
        }
        // second
        238 | 18 => {
            let hh = abs_unit / 3600;
            let mm = (abs_unit / 60) % 60;
            let ss = abs_unit % 60;
            Some(format!("{sign}{hh:02}:{mm:02}:{ss:02}"))
        }
        // time (ms)
        237 | 19 => {
            let hh = abs_unit / 3_600_000;
            let mm = (abs_unit / 60_000) % 60;
            let ss = (abs_unit / 1000) % 60;
            let ms = abs_unit % 1000;
            Some(format!("{sign}{hh:02}:{mm:02}:{ss:02}.{ms:03}"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timespan_positive() {
        assert_eq!(
            bigint_to_timespan(21_110_218_577_000),
            Some("0D05:51:50.218577000".to_string())
        );
    }

    #[test]
    fn test_timespan_negative() {
        assert_eq!(
            bigint_to_timespan(-21_110_218_577_000),
            Some("-0D05:51:50.218577000".to_string())
        );
    }

    #[test]
    fn test_timespan_null() {
        assert_eq!(bigint_to_timespan(LONG_NULL), None);
    }

    #[test]
    fn test_month() {
        assert_eq!(int_to_temporal(273, 13), Some("2022.10m".to_string()));
        assert_eq!(int_to_temporal(-147, 13), Some("1987.10m".to_string()));
        assert_eq!(int_to_temporal(0, 13), Some("2000.01m".to_string()));
        assert_eq!(int_to_temporal(-1, 13), Some("1999.12m".to_string()));
        assert_eq!(int_to_temporal(-12, 13), Some("1999.01m".to_string()));
    }

    #[test]
    fn test_minute() {
        assert_eq!(int_to_temporal(851, 17), Some("14:11".to_string()));
        assert_eq!(int_to_temporal(-9, 17), Some("-00:09".to_string()));
        assert_eq!(int_to_temporal(-851, 17), Some("-14:11".to_string()));
    }

    #[test]
    fn test_second() {
        assert_eq!(int_to_temporal(51109, 18), Some("14:11:49".to_string()));
        assert_eq!(int_to_temporal(-9, 18), Some("-00:00:09".to_string()));
        assert_eq!(int_to_temporal(-51109, 18), Some("-14:11:49".to_string()));
    }

    #[test]
    fn test_time_ms() {
        assert_eq!(
            int_to_temporal(51109668, 19),
            Some("14:11:49.668".to_string())
        );
        assert_eq!(int_to_temporal(-9, 19), Some("-00:00:00.009".to_string()));
        assert_eq!(
            int_to_temporal(-51109668, 19),
            Some("-14:11:49.668".to_string())
        );
    }

    #[test]
    fn test_null_sentinels() {
        assert_eq!(int_to_temporal(INT_NULL, 13), None);
        assert_eq!(int_to_temporal(INT_POS_INF, 17), None);
        assert_eq!(int_to_temporal(INT_NEG_INF, 18), None);
    }
}
