//! A minimal UTC-offset date and time, covering only what `smolgen` needs.

use crate::Result;
use std::time::{SystemTime, UNIX_EPOCH};

const SECS_PER_DAY: i64 = 86_400;
const MIN_YEAR: i64 = -9999;
const MAX_YEAR: i64 = 9999;
const MIN_UNIX: i64 = days_from_civil(MIN_YEAR, 1, 1) * SECS_PER_DAY;
const MAX_UNIX: i64 = days_from_civil(MAX_YEAR, 12, 31) * SECS_PER_DAY + SECS_PER_DAY - 1;

/// An instant with nanosecond precision and the UTC offset it is displayed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DateTime {
    unix: i64,
    nanos: u32,
    offset: i32,
}

impl DateTime {
    /// The current time in UTC.
    pub(crate) fn now_utc() -> Self {
        let (unix, nanos) = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => (d.as_secs() as i64, d.subsec_nanos()),
            Err(e) => {
                let d = e.duration();
                match d.subsec_nanos() {
                    0 => (-(d.as_secs() as i64), 0),
                    n => (-(d.as_secs() as i64) - 1, 1_000_000_000 - n),
                }
            }
        };
        Self {
            unix,
            nanos,
            offset: 0,
        }
    }

    /// The UTC instant `unix` seconds after the Unix epoch, for years -9999 through 9999.
    pub(crate) fn from_unix_timestamp(unix: i64) -> Result<Self> {
        if !(MIN_UNIX..=MAX_UNIX).contains(&unix) {
            return Err(format!("unix timestamp {unix} out of range").into());
        }
        Ok(Self {
            unix,
            nanos: 0,
            offset: 0,
        })
    }

    /// Parses an RFC 3339 timestamp such as `2024-01-31T03:26:34+01:00`.
    #[cfg_attr(not(feature = "gitcl"), allow(dead_code))]
    pub(crate) fn parse_rfc3339(s: &str) -> Result<Self> {
        Self::parse_rfc3339_bytes(s.as_bytes())
            .ok_or_else(|| format!("invalid RFC 3339 timestamp: {s:?}").into())
    }

    fn parse_rfc3339_bytes(b: &[u8]) -> Option<Self> {
        fn num(b: &[u8], range: std::ops::Range<usize>) -> Option<i64> {
            b.get(range)?.iter().try_fold(0, |acc, &c| {
                c.is_ascii_digit().then(|| acc * 10 + i64::from(c - b'0'))
            })
        }

        let year = num(b, 0..4)?;
        let month = num(b, 5..7)?;
        let day = num(b, 8..10)?;
        let hour = num(b, 11..13)?;
        let minute = num(b, 14..16)?;
        let mut second = num(b, 17..19)?;
        // Any single byte may separate the date and the time.
        if b[4] != b'-' || b[7] != b'-' || b[13] != b':' || b[16] != b':' {
            return None;
        }

        let mut rest = &b[19..];
        let mut nanos = 0;
        if let [b'.', tail @ ..] = rest {
            let digits = tail.iter().take_while(|c| c.is_ascii_digit()).count();
            if digits == 0 {
                return None;
            }
            nanos = tail[..digits.min(9)]
                .iter()
                .fold(0, |acc, &c| acc * 10 + u32::from(c - b'0'))
                * 10u32.pow(9 - digits.min(9) as u32);
            rest = &tail[digits..];
        }

        let offset = match rest {
            [b'Z' | b'z'] => 0,
            [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
                let (h, m) = (num(rest, 1..3)?, num(rest, 4..6)?);
                if h > 23 || m > 59 {
                    return None;
                }
                let offset = (h * 3600 + m * 60) as i32;
                if *sign == b'-' { -offset } else { offset }
            }
            _ => return None,
        };

        let leap_second = second == 60;
        if leap_second {
            second = 59;
            nanos = 999_999_999;
        }
        if !(1..=12).contains(&month)
            || !(1..=days_in_month(year, month)).contains(&day)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return None;
        }

        let local =
            days_from_civil(year, month, day) * SECS_PER_DAY + hour * 3600 + minute * 60 + second;
        let dt = Self {
            unix: local - i64::from(offset),
            nanos,
            offset,
        };
        if leap_second {
            // A leap second can only occur as the last second of a month in UTC.
            let (y, m, d) = civil_from_days(dt.unix.div_euclid(SECS_PER_DAY));
            if dt.unix.rem_euclid(SECS_PER_DAY) != SECS_PER_DAY - 1 || d != days_in_month(y, m) {
                return None;
            }
        }
        Some(dt)
    }

    /// Converts to the system's current local UTC offset.
    ///
    /// The value is unchanged if the local date would fall outside years -9999 through 9999.
    pub(crate) fn to_local(self) -> Result<Self> {
        let local = Self {
            offset: local_offset()?,
            ..self
        };
        let (year, ..) = local.civil();
        Ok(if (MIN_YEAR..=MAX_YEAR).contains(&year) {
            local
        } else {
            self
        })
    }

    /// Seconds since the Unix epoch.
    pub(crate) fn unix_timestamp(self) -> i64 {
        self.unix
    }

    /// Formats the date in this value's offset as `YYYY-MM-DD`.
    pub(crate) fn format_date(self) -> String {
        let (year, month, day, _) = self.civil();
        let sign = if year < 0 { "-" } else { "" };
        format!("{sign}{:04}-{month:02}-{day:02}", year.unsigned_abs())
    }

    /// Formats the value in its offset as ISO 8601, for example
    /// `2024-01-31T03:26:34.065893658Z` or `2026-10-02T09:14:57.000000000-04:00`.
    pub(crate) fn format_iso8601(self) -> Result<String> {
        let (year, month, day, secs) = self.civil();
        if year < 0 {
            return Err(format!("year {year} cannot be formatted as ISO 8601").into());
        }
        let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
        let mut out = format!(
            "{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}.{:09}",
            self.nanos
        );
        if self.offset == 0 {
            out.push('Z');
        } else {
            let sign = if self.offset < 0 { '-' } else { '+' };
            let abs = self.offset.unsigned_abs();
            out.push_str(&format!("{sign}{:02}:{:02}", abs / 3600, abs / 60 % 60));
        }
        Ok(out)
    }

    /// The local year, month, day and seconds into the day.
    fn civil(self) -> (i64, i64, i64, i64) {
        let local = self.unix + i64::from(self.offset);
        let (y, m, d) = civil_from_days(local.div_euclid(SECS_PER_DAY));
        (y, m, d, local.rem_euclid(SECS_PER_DAY))
    }
}

#[cfg(unix)]
fn local_offset() -> Result<i32> {
    let output = std::process::Command::new("date").arg("+%z").output()?;
    if !output.status.success() {
        return Err("`date +%z` failed".into());
    }
    let out = String::from_utf8_lossy(&output.stdout);
    let out = out.trim();
    let (sign, digits) = match out.as_bytes() {
        [b'+', rest @ ..] => (1, rest),
        [b'-', rest @ ..] => (-1, rest),
        _ => (0, &[][..]),
    };
    match digits {
        [h1, h2, m1, m2] if digits.iter().all(u8::is_ascii_digit) => {
            let h = i32::from((h1 - b'0') * 10 + (h2 - b'0'));
            let m = i32::from((m1 - b'0') * 10 + (m2 - b'0'));
            if sign != 0 && m < 60 {
                return Ok(sign * (h * 3600 + m * 60));
            }
        }
        _ => {}
    }
    Err(format!("unexpected `date +%z` output: {out:?}").into())
}

#[cfg(not(unix))]
fn local_offset() -> Result<i32> {
    Err("the local UTC offset is only available on Unix".into())
}

const fn is_leap_year(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

const fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        2 if is_leap_year(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's `days_from_civil`).
const fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The proleptic Gregorian date of a day since 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unix_timestamp_range() {
        assert!(DateTime::from_unix_timestamp(MIN_UNIX).is_ok());
        assert!(DateTime::from_unix_timestamp(MAX_UNIX).is_ok());
        assert!(DateTime::from_unix_timestamp(MIN_UNIX - 1).is_err());
        assert!(DateTime::from_unix_timestamp(MAX_UNIX + 1).is_err());
    }

    #[test]
    fn rfc3339_rejects_invalid() {
        for s in [
            "",
            "2024-01-31",
            "2024-01-31T03:26:34",
            "2024-13-01T00:00:00Z",
            "2023-02-29T00:00:00Z",
            "2024-01-31T24:00:00Z",
            "2024-01-31T03:26:34.Z",
            "2024-01-31T03:26:34+24:00",
            "2024-01-31T03:26:34+01:60",
            "2024-01-31T03:26:34+0100",
            "2024-01-31T03:26:34Z\n",
            "2024-01-30T23:59:60Z",
        ] {
            assert!(DateTime::parse_rfc3339(s).is_err(), "{s:?}");
        }
    }
}
