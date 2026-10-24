//! A minimal UTC-offset date and time, covering only what `smolgen` needs.

#![cfg_attr(
    not(all(feature = "build", feature = "gitcl")),
    allow(
        dead_code,
        reason = "each feature uses a different part of this module"
    )
)]

use crate::{Error, Result};
use std::time::{SystemTime, UNIX_EPOCH};

const SECS_PER_DAY: i64 = 86_400;
const MIN_YEAR: i64 = -9999;
const MAX_YEAR: i64 = 9999;
const MIN_UNIX: i64 = days_from_civil(MIN_YEAR, 1, 1) * SECS_PER_DAY;
const MAX_UNIX: i64 = days_from_civil(MAX_YEAR, 12, 31) * SECS_PER_DAY + SECS_PER_DAY - 1;
/// The largest UTC offset `time` accepts, 25:59:59.
const MAX_OFFSET: i32 = 93_599;

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
            return Err(Error::msg(format!("unix timestamp {unix} out of range")));
        }
        Ok(Self {
            unix,
            nanos: 0,
            offset: 0,
        })
    }

    /// Parses an RFC 3339 timestamp such as `2024-01-31T03:26:34+01:00`.
    pub(crate) fn parse_rfc3339(s: &str) -> Result<Self> {
        Self::parse_rfc3339_bytes(s.as_bytes())
            .ok_or_else(|| Error::msg(format!("invalid RFC 3339 timestamp: {s:?}")))
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

    /// Converts to the system's local UTC offset at this instant.
    ///
    /// The value is unchanged if the local date would fall outside years -9999 through 9999.
    pub(crate) fn to_local(self) -> Result<Self> {
        let offset = sys::local_offset_at(self)
            .filter(|offset| offset.abs() <= MAX_OFFSET)
            .ok_or_else(|| Error::msg("unable to determine the local UTC offset"))?;
        let local = Self { offset, ..self };
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
            return Err(Error::msg(format!(
                "year {year} cannot be formatted as ISO 8601"
            )));
        }
        if self.offset % 60 != 0 {
            return Err(Error::msg(
                "UTC offsets with seconds cannot be formatted as ISO 8601",
            ));
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

/// Reads the local UTC offset from `struct tm`'s `tm_gmtoff`, which follows the nine `c_int`
/// fields on these targets.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly",
))]
mod sys {
    use super::DateTime;
    use std::{
        ffi::{c_char, c_int, c_long},
        mem::MaybeUninit,
    };

    #[cfg(any(
        target_os = "netbsd",
        target_os = "openbsd",
        all(target_os = "freebsd", not(target_arch = "x86")),
    ))]
    type TimeT = i64;
    #[cfg(not(any(
        target_os = "netbsd",
        target_os = "openbsd",
        all(target_os = "freebsd", not(target_arch = "x86")),
    )))]
    type TimeT = c_long;

    #[repr(C)]
    struct Tm {
        _fields: [c_int; 9],
        tm_gmtoff: c_long,
        _tm_zone: *const c_char,
    }

    unsafe extern "C" {
        #[cfg_attr(target_os = "netbsd", link_name = "__localtime_r50")]
        fn localtime_r(time: *const TimeT, result: *mut Tm) -> *mut Tm;
    }

    pub(super) fn local_offset_at(dt: DateTime) -> Option<i32> {
        #[allow(
            clippy::useless_conversion,
            reason = "`TimeT` is `i64` on some targets"
        )]
        let time = TimeT::try_from(dt.unix).ok()?;
        let mut tm = MaybeUninit::<Tm>::uninit();
        // SAFETY: `time` and `tm` are valid for reads and writes, and `Tm` covers the platform's
        // `struct tm`. `localtime_r` returns null on failure.
        if unsafe { localtime_r(&time, tm.as_mut_ptr()) }.is_null() {
            return None;
        }
        // SAFETY: `localtime_r` succeeded, so it initialized `tm`.
        let tm = unsafe { tm.assume_init() };
        #[allow(
            clippy::useless_conversion,
            reason = "`c_long` is `i32` on some targets"
        )]
        i32::try_from(tm.tm_gmtoff).ok()
    }
}

/// Compares a UTC `SYSTEMTIME` with its conversion to the current time zone, as `time` does.
#[cfg(windows)]
mod sys {
    use super::DateTime;
    use std::{ffi::c_void, mem::MaybeUninit};

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct SystemTime {
        year: u16,
        month: u16,
        day_of_week: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milliseconds: u16,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SystemTimeToFileTime(system_time: *const SystemTime, file_time: *mut FileTime) -> i32;
        fn SystemTimeToTzSpecificLocalTime(
            time_zone: *const c_void,
            universal_time: *const SystemTime,
            local_time: *mut SystemTime,
        ) -> i32;
    }

    /// Whole seconds since 1601-01-01.
    fn secs(system_time: &SystemTime) -> Option<i64> {
        let mut ft = MaybeUninit::<FileTime>::uninit();
        // SAFETY: Both pointers are valid, and the call returns zero on failure.
        if unsafe { SystemTimeToFileTime(system_time, ft.as_mut_ptr()) } == 0 {
            return None;
        }
        // SAFETY: The call succeeded, so it initialized `ft`.
        let ft = unsafe { ft.assume_init() };
        Some(((u64::from(ft.high) << 32 | u64::from(ft.low)) / 10_000_000) as i64)
    }

    pub(super) fn local_offset_at(dt: DateTime) -> Option<i32> {
        let (year, month, day, sod) = DateTime { offset: 0, ..dt }.civil();
        let utc = SystemTime {
            year: year as u16,
            month: month as u16,
            day_of_week: 0,
            day: day as u16,
            hour: (sod / 3600) as u16,
            minute: (sod / 60 % 60) as u16,
            second: (sod % 60) as u16,
            milliseconds: (dt.nanos / 1_000_000) as u16,
        };
        let mut local = MaybeUninit::<SystemTime>::uninit();
        // SAFETY: A null time zone selects the current one, the other pointers are valid, and the
        // call returns zero on failure.
        if unsafe { SystemTimeToTzSpecificLocalTime(std::ptr::null(), &utc, local.as_mut_ptr()) }
            == 0
        {
            return None;
        }
        // SAFETY: The call succeeded, so it initialized `local`.
        let local = unsafe { local.assume_init() };
        i32::try_from(secs(&local)? - secs(&utc)?).ok()
    }
}

#[cfg(not(any(
    windows,
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly",
)))]
mod sys {
    pub(super) fn local_offset_at(_: super::DateTime) -> Option<i32> {
        None
    }
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
