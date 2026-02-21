//! Time humanizing functions.
//!
//! These are largely borrowed from Django's `contrib.humanize`.

use crate::i18n;
use crate::number::{intcomma, printf_format};
use chrono::{Local, NaiveDate};
use std::collections::HashSet;

/// Unit enum for time precision, ordered from smallest to largest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unit {
    Microseconds = 0,
    Milliseconds = 1,
    Seconds = 2,
    Minutes = 3,
    Hours = 4,
    Days = 5,
    Months = 6,
    Years = 7,
}

impl Unit {
    /// Parse a unit name (case-insensitive).
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "MICROSECONDS" => Ok(Unit::Microseconds),
            "MILLISECONDS" => Ok(Unit::Milliseconds),
            "SECONDS" => Ok(Unit::Seconds),
            "MINUTES" => Ok(Unit::Minutes),
            "HOURS" => Ok(Unit::Hours),
            "DAYS" => Ok(Unit::Days),
            "MONTHS" => Ok(Unit::Months),
            "YEARS" => Ok(Unit::Years),
            _ => Err(format!("Unknown unit: {}", s)),
        }
    }

    fn all() -> &'static [Unit] {
        &[
            Unit::Microseconds,
            Unit::Milliseconds,
            Unit::Seconds,
            Unit::Minutes,
            Unit::Hours,
            Unit::Days,
            Unit::Months,
            Unit::Years,
        ]
    }

}

/// A duration broken into days, seconds, and microseconds (like Python's timedelta).
#[derive(Debug, Clone, Copy)]
pub struct TimeDelta {
    pub days: i64,
    pub seconds: i64,
    pub microseconds: i64,
}

impl TimeDelta {
    /// Create from total seconds (float).
    pub fn from_seconds(secs: f64) -> Self {
        let total_us = (secs * 1_000_000.0).round() as i128;
        let total_us_abs = total_us.unsigned_abs();
        let sign: i64 = if total_us < 0 { -1 } else { 1 };

        let days = (total_us_abs / (86_400 * 1_000_000)) as i64 * sign;
        let remaining_us = (total_us_abs % (86_400 * 1_000_000)) as i64;
        let seconds = (remaining_us / 1_000_000) * if total_us < 0 && remaining_us > 0 { 1 } else { 1 };
        let microseconds = remaining_us % 1_000_000;

        // Match Python's timedelta normalization where days can be negative
        // but seconds and microseconds are always 0 <= x < max
        if total_us < 0 {
            let abs_days = (total_us_abs / (86_400 * 1_000_000)) as i64;
            let remaining = total_us_abs % (86_400 * 1_000_000);
            if remaining > 0 {
                let d = -(abs_days + 1);
                let remaining_positive = 86_400 * 1_000_000u128 - remaining;
                let s = (remaining_positive / 1_000_000) as i64;
                let us = (remaining_positive % 1_000_000) as i64;
                TimeDelta { days: d, seconds: s, microseconds: us }
            } else {
                TimeDelta { days: -abs_days, seconds: 0, microseconds: 0 }
            }
        } else {
            TimeDelta { days, seconds, microseconds }
        }
    }

    pub fn from_days_seconds_micros(days: i64, seconds: i64, microseconds: i64) -> Self {
        // Normalize like Python's timedelta
        let total_us = days as i128 * 86_400_000_000i128
            + seconds as i128 * 1_000_000i128
            + microseconds as i128;

        if total_us < 0 {
            let abs_us = (-total_us) as u128;
            let abs_days = (abs_us / 86_400_000_000) as i64;
            let remaining = abs_us % 86_400_000_000;
            if remaining > 0 {
                let d = -(abs_days + 1);
                let remaining_positive = 86_400_000_000u128 - remaining;
                let s = (remaining_positive / 1_000_000) as i64;
                let us = (remaining_positive % 1_000_000) as i64;
                TimeDelta { days: d, seconds: s, microseconds: us }
            } else {
                TimeDelta { days: -abs_days, seconds: 0, microseconds: 0 }
            }
        } else {
            let total_us = total_us as u128;
            let days = (total_us / 86_400_000_000) as i64;
            let remaining = total_us % 86_400_000_000;
            let seconds = (remaining / 1_000_000) as i64;
            let microseconds = (remaining % 1_000_000) as i64;
            TimeDelta { days, seconds, microseconds }
        }
    }

    /// Total seconds as float.
    pub fn total_seconds(&self) -> f64 {
        self.days as f64 * 86_400.0
            + self.seconds as f64
            + self.microseconds as f64 / 1_000_000.0
    }

    /// Absolute value.
    pub fn abs(&self) -> Self {
        if self.days < 0 {
            let total_us = self.days.unsigned_abs() as i128 * 86_400_000_000i128
                - self.seconds as i128 * 1_000_000i128
                - self.microseconds as i128;
            let total_us = total_us.unsigned_abs();
            let days = (total_us / 86_400_000_000) as i64;
            let remaining = total_us % 86_400_000_000;
            let seconds = (remaining / 1_000_000) as i64;
            let microseconds = (remaining % 1_000_000) as i64;
            TimeDelta { days, seconds, microseconds }
        } else {
            *self
        }
    }
}

/// Return a natural representation of a timedelta or number of seconds.
///
/// This does not add tense to the result.
///
/// # Examples
/// ```
/// use speakhuman::time::{naturaldelta_td, TimeDelta};
/// let delta = TimeDelta::from_days_seconds_micros(7, 0, 0);
/// assert_eq!(naturaldelta_td(delta, true, "seconds"), "7 days");
/// ```
pub fn naturaldelta_td(value: TimeDelta, months: bool, minimum_unit: &str) -> String {
    let min_unit = match Unit::from_str(minimum_unit) {
        Ok(u) => u,
        Err(e) => return e,
    };

    if min_unit != Unit::Seconds && min_unit != Unit::Milliseconds && min_unit != Unit::Microseconds
    {
        return format!("Minimum unit '{}' not supported", minimum_unit);
    }

    let delta = value.abs();
    let years = delta.days / 365;
    let days = delta.days % 365;
    let num_months = ((days as f64) / 30.5).round() as i64;

    if years == 0 && days < 1 {
        if delta.seconds == 0 {
            if min_unit == Unit::Microseconds && delta.microseconds < 1000 {
                let us = delta.microseconds;
                let template = i18n::ngettext("%d microsecond", "%d microseconds", us);
                return template.replace("%d", &us.to_string());
            }

            if min_unit == Unit::Milliseconds
                || (min_unit == Unit::Microseconds
                    && delta.microseconds >= 1000
                    && delta.microseconds < 1_000_000)
            {
                let ms = delta.microseconds / 1000;
                let template = i18n::ngettext("%d millisecond", "%d milliseconds", ms);
                return template.replace("%d", &ms.to_string());
            }

            return i18n::gettext("a moment");
        }

        if delta.seconds == 1 {
            return i18n::gettext("a second");
        }

        if delta.seconds < 60 {
            let s = delta.seconds;
            let template = i18n::ngettext("%d second", "%d seconds", s);
            return template.replace("%d", &s.to_string());
        }

        if delta.seconds >= 60 && delta.seconds < 3600 {
            let minutes = ((delta.seconds as f64) / 60.0).round() as i64;
            if minutes == 1 {
                return i18n::gettext("a minute");
            }
            if minutes == 60 {
                return i18n::gettext("an hour");
            }
            let template = i18n::ngettext("%d minute", "%d minutes", minutes);
            return template.replace("%d", &minutes.to_string());
        }

        if delta.seconds >= 3600 {
            let hours = ((delta.seconds as f64) / 3600.0).round() as i64;
            if hours == 1 {
                return i18n::gettext("an hour");
            }
            if hours == 24 {
                return i18n::gettext("a day");
            }
            let template = i18n::ngettext("%d hour", "%d hours", hours);
            return template.replace("%d", &hours.to_string());
        }
    } else if years == 0 {
        if days == 1 {
            return i18n::gettext("a day");
        }

        if !months {
            let template = i18n::ngettext("%d day", "%d days", days);
            return template.replace("%d", &days.to_string());
        }

        if num_months == 0 {
            let template = i18n::ngettext("%d day", "%d days", days);
            return template.replace("%d", &days.to_string());
        }

        if num_months == 1 {
            return i18n::gettext("a month");
        }

        if num_months == 12 {
            return i18n::gettext("a year");
        }

        let template = i18n::ngettext("%d month", "%d months", num_months);
        return template.replace("%d", &num_months.to_string());
    } else if years == 1 {
        if num_months == 0 && days == 0 {
            return i18n::gettext("a year");
        }

        if num_months == 0 {
            let template = i18n::ngettext("1 year, %d day", "1 year, %d days", days);
            return template.replace("%d", &days.to_string());
        }

        if months {
            if num_months == 1 {
                return i18n::gettext("1 year, 1 month");
            }

            if num_months == 12 {
                let y = years + 1;
                let template = i18n::ngettext("%d year", "%d years", y);
                return template.replace("%d", &y.to_string());
            }

            let template =
                i18n::ngettext("1 year, %d month", "1 year, %d months", num_months);
            return template.replace("%d", &num_months.to_string());
        }

        let template = i18n::ngettext("1 year, %d day", "1 year, %d days", days);
        return template.replace("%d", &days.to_string());
    }

    // years >= 2
    let template = i18n::ngettext("%d year", "%d years", years);
    let with_placeholder = template.replace("%d", "%s");
    with_placeholder.replace("%s", &intcomma(&years.to_string(), None))
}

/// Convenience: naturaldelta from seconds (float).
pub fn naturaldelta(seconds: f64, months: bool, minimum_unit: &str) -> String {
    let delta = TimeDelta::from_seconds(seconds);
    naturaldelta_td(delta, months, minimum_unit)
}

/// Return a natural representation of a time, with tense.
///
/// # Arguments
/// * `delta` - A TimeDelta representing the time difference.
/// * `future` - Whether the time is in the future.
/// * `months` - Whether to use month approximations.
/// * `minimum_unit` - The minimum unit to display.
pub fn naturaltime_delta(
    delta: TimeDelta,
    future: bool,
    months: bool,
    minimum_unit: &str,
) -> String {
    let delta_str = naturaldelta_td(delta, months, minimum_unit);

    if delta_str == i18n::gettext("a moment") {
        return i18n::gettext("now");
    }

    let ago_template = if future {
        i18n::gettext("%s from now")
    } else {
        i18n::gettext("%s ago")
    };

    ago_template.replace("%s", &delta_str)
}

/// Return "today", "tomorrow", or "yesterday" for nearby dates,
/// otherwise format with the given strftime format.
pub fn naturalday(value: NaiveDate, format: &str) -> String {
    let today = Local::now().date_naive();
    let diff = (value - today).num_days();

    if diff == 0 {
        return i18n::gettext("today");
    }
    if diff == 1 {
        return i18n::gettext("tomorrow");
    }
    if diff == -1 {
        return i18n::gettext("yesterday");
    }

    value.format(format).to_string()
}

/// Like naturalday, but append a year for dates more than ~five months away.
pub fn naturaldate(value: NaiveDate) -> String {
    let today = Local::now().date_naive();
    let diff = (value - today).num_days().unsigned_abs();

    if diff >= (5 * 365 / 12) as u64 {
        naturalday(value, "%b %d %Y")
    } else {
        naturalday(value, "%b %d")
    }
}

/// Divide value by divisor with special handling for minimum_unit and suppressed units.
fn quotient_and_remainder(
    value: f64,
    divisor: f64,
    unit: Unit,
    minimum_unit: Unit,
    suppress: &HashSet<Unit>,
    format: &str,
) -> (f64, f64) {
    if unit == minimum_unit {
        return (rounding_by_fmt(format, value / divisor), 0.0);
    }

    if suppress.contains(&unit) {
        return (0.0, value);
    }

    let q = (value / divisor).floor();
    let r = value - q * divisor;
    (q, r.floor().max(0.0))
}

/// Find a suitable minimum unit that is not suppressed.
fn suitable_minimum_unit(min_unit: Unit, suppress: &HashSet<Unit>) -> Result<Unit, String> {
    if !suppress.contains(&min_unit) {
        return Ok(min_unit);
    }
    for unit in Unit::all() {
        if *unit > min_unit && !suppress.contains(unit) {
            return Ok(*unit);
        }
    }
    Err("Minimum unit is suppressed and no suitable replacement was found".to_string())
}

/// Extend suppressed units with all units lower than the minimum unit.
fn suppress_lower_units(min_unit: Unit, suppress: &HashSet<Unit>) -> HashSet<Unit> {
    let mut result = suppress.clone();
    for unit in Unit::all() {
        if *unit == min_unit {
            break;
        }
        result.insert(*unit);
    }
    result
}

/// Round a number according to printf-style format string.
fn rounding_by_fmt(format: &str, value: f64) -> f64 {
    let result = printf_format(format, value);
    result
        .trim()
        .parse::<i64>()
        .map(|i| i as f64)
        .unwrap_or_else(|_| result.trim().parse::<f64>().unwrap_or(value))
}

/// Return a precise representation of a timedelta.
///
/// # Examples
/// ```
/// use speakhuman::time::{precisedelta_td, TimeDelta};
/// let delta = TimeDelta::from_days_seconds_micros(2, 3633, 123000);
/// assert_eq!(precisedelta_td(delta, "seconds", &[], "%0.2f"), "2 days, 1 hour and 33.12 seconds");
/// ```
pub fn precisedelta_td(
    value: TimeDelta,
    minimum_unit: &str,
    suppress: &[&str],
    format: &str,
) -> String {
    let delta = value.abs();

    let suppress_set: HashSet<Unit> = suppress
        .iter()
        .filter_map(|s| Unit::from_str(s).ok())
        .collect();

    let min_unit = match Unit::from_str(minimum_unit) {
        Ok(u) => u,
        Err(e) => return e,
    };
    let min_unit = match suitable_minimum_unit(min_unit, &suppress_set) {
        Ok(u) => u,
        Err(e) => return e,
    };

    let suppress_set = suppress_lower_units(min_unit, &suppress_set);

    let days = delta.days as f64;
    let secs = delta.seconds as f64;
    let usecs = delta.microseconds as f64;

    let (years, days) = quotient_and_remainder(
        days,
        365.0,
        Unit::Years,
        min_unit,
        &suppress_set,
        format,
    );
    let (months, days) = quotient_and_remainder(
        days,
        30.5,
        Unit::Months,
        min_unit,
        &suppress_set,
        format,
    );

    let secs = days * 24.0 * 3600.0 + secs;
    let (days, secs) = quotient_and_remainder(
        secs,
        24.0 * 3600.0,
        Unit::Days,
        min_unit,
        &suppress_set,
        format,
    );

    let (hours, secs) = quotient_and_remainder(
        secs,
        3600.0,
        Unit::Hours,
        min_unit,
        &suppress_set,
        format,
    );
    let (minutes, secs) = quotient_and_remainder(
        secs,
        60.0,
        Unit::Minutes,
        min_unit,
        &suppress_set,
        format,
    );

    let usecs = secs * 1e6 + usecs;
    let (secs, usecs) = quotient_and_remainder(
        usecs,
        1e6,
        Unit::Seconds,
        min_unit,
        &suppress_set,
        format,
    );

    let (msecs, usecs) = quotient_and_remainder(
        usecs,
        1000.0,
        Unit::Milliseconds,
        min_unit,
        &suppress_set,
        format,
    );

    // Promotion due to rounding
    let (mut msecs, mut secs, mut minutes, mut hours, mut days, mut months, mut years) =
        (msecs, secs, minutes, hours, days, months, years);

    if msecs >= 1000.0 && !suppress_set.contains(&Unit::Seconds) {
        msecs -= 1000.0;
        secs += 1.0;
    }
    if secs >= 60.0 && !suppress_set.contains(&Unit::Minutes) {
        secs -= 60.0;
        minutes += 1.0;
    }
    if minutes >= 60.0 && !suppress_set.contains(&Unit::Hours) {
        minutes -= 60.0;
        hours += 1.0;
    }
    if hours >= 24.0 && !suppress_set.contains(&Unit::Days) {
        hours -= 24.0;
        days += 1.0;
    }
    if days >= 31.0 && !suppress_set.contains(&Unit::Months) {
        days -= 31.0;
        months += 1.0;
    }
    if months >= 12.0 && !suppress_set.contains(&Unit::Years) {
        months -= 12.0;
        years += 1.0;
    }

    let fmts: Vec<(&str, &str, f64, Unit)> = vec![
        ("%d year", "%d years", years, Unit::Years),
        ("%d month", "%d months", months, Unit::Months),
        ("%d day", "%d days", days, Unit::Days),
        ("%d hour", "%d hours", hours, Unit::Hours),
        ("%d minute", "%d minutes", minutes, Unit::Minutes),
        ("%d second", "%d seconds", secs, Unit::Seconds),
        ("%d millisecond", "%d milliseconds", msecs, Unit::Milliseconds),
        (
            "%d microsecond",
            "%d microseconds",
            usecs,
            Unit::Microseconds,
        ),
    ];

    let mut texts: Vec<String> = Vec::new();

    for (singular, plural, fmt_value, unit) in fmts.iter() {
        let unit = *unit;
        let fmt_value = *fmt_value;

        if fmt_value > 0.0 || (texts.is_empty() && unit == min_unit) {
            let ngettext_n = if fmt_value > 1.0 && fmt_value < 2.0 {
                2
            } else {
                fmt_value as i64
            };
            let mut fmt_txt = i18n::ngettext(singular, plural, ngettext_n);

            let frac = fmt_value - (fmt_value as i64) as f64;
            if unit == min_unit && frac.abs() > 1e-9 {
                fmt_txt = fmt_txt.replace("%d", format);
            } else if unit == Unit::Years {
                let int_check = fmt_value - (fmt_value as i64) as f64;
                let display_val = if int_check.abs() < 1e-9 {
                    fmt_value as i64
                } else {
                    fmt_value as i64
                };
                fmt_txt = fmt_txt.replace("%d", "%s");
                let formatted = fmt_txt.replace("%s", &intcomma(&display_val.to_string(), None));
                texts.push(formatted);
                if unit == min_unit {
                    break;
                }
                continue;
            }

            let formatted = if unit == min_unit && frac.abs() > 1e-9 {
                // The format string was substituted into fmt_txt (e.g., "%0.2f seconds")
                // We need to format the number part and keep the text
                let number_str = printf_format(format, fmt_value);
                fmt_txt.replace(format, &number_str)
            } else {
                fmt_txt.replace("%d", &(fmt_value as i64).to_string())
            };

            texts.push(formatted);
        }

        if unit == min_unit {
            break;
        }
    }

    if texts.len() <= 1 {
        return texts.into_iter().next().unwrap_or_default();
    }

    let head = texts[..texts.len() - 1].join(", ");
    let tail = &texts[texts.len() - 1];

    let template = i18n::gettext("%s and %s");
    // Replace first %s with head, second %s with tail
    let result = template.replacen("%s", &head, 1);
    result.replacen("%s", tail, 1)
}

/// Convenience: precisedelta from seconds (float).
pub fn precisedelta(seconds: f64, minimum_unit: &str, suppress: &[&str], format: &str) -> String {
    let delta = TimeDelta::from_seconds(seconds);
    precisedelta_td(delta, minimum_unit, suppress, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_ordering() {
        assert!(Unit::Seconds < Unit::Minutes);
        assert!(Unit::Minutes < Unit::Hours);
        assert!(Unit::Years > Unit::Days);
    }

    #[test]
    fn test_timedelta_from_seconds() {
        let td = TimeDelta::from_seconds(90.0);
        assert_eq!(td.days, 0);
        assert_eq!(td.seconds, 90);
        assert_eq!(td.microseconds, 0);
    }

    #[test]
    fn test_naturaldelta_basic() {
        let td = TimeDelta::from_days_seconds_micros(7, 0, 0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "7 days");
    }

    #[test]
    fn test_naturaldelta_moment() {
        let td = TimeDelta::from_seconds(0.0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "a moment");
    }

    #[test]
    fn test_naturaldelta_seconds() {
        let td = TimeDelta::from_seconds(1.0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "a second");

        let td = TimeDelta::from_seconds(30.0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "30 seconds");
    }

    #[test]
    fn test_naturaldelta_minutes() {
        let td = TimeDelta::from_seconds(120.0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "2 minutes");
    }

    #[test]
    fn test_naturaldelta_hours() {
        let td = TimeDelta::from_days_seconds_micros(0, 3600, 0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "an hour");
    }

    #[test]
    fn test_naturaldelta_years() {
        let td = TimeDelta::from_days_seconds_micros(365 * 2 + 35, 0, 0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "2 years");
    }

    #[test]
    fn test_naturaldelta_large_years() {
        let td = TimeDelta::from_days_seconds_micros(365 * 1141, 0, 0);
        assert_eq!(naturaldelta_td(td, true, "seconds"), "1,141 years");
    }

    #[test]
    fn test_naturaltime_past() {
        let td = TimeDelta::from_seconds(30.0);
        assert_eq!(
            naturaltime_delta(td, false, true, "seconds"),
            "30 seconds ago"
        );
    }

    #[test]
    fn test_naturaltime_future() {
        let td = TimeDelta::from_seconds(30.0);
        assert_eq!(
            naturaltime_delta(td, true, true, "seconds"),
            "30 seconds from now"
        );
    }

    #[test]
    fn test_naturaltime_now() {
        let td = TimeDelta::from_seconds(0.0);
        assert_eq!(naturaltime_delta(td, false, true, "seconds"), "now");
    }

    #[test]
    fn test_precisedelta_basic() {
        let td = TimeDelta::from_days_seconds_micros(2, 3633, 123000);
        assert_eq!(
            precisedelta_td(td, "seconds", &[], "%0.2f"),
            "2 days, 1 hour and 33.12 seconds"
        );
    }

    #[test]
    fn test_precisedelta_single_unit() {
        let td = TimeDelta::from_seconds(1.0);
        assert_eq!(precisedelta_td(td, "seconds", &[], "%0.2f"), "1 second");

        let td = TimeDelta::from_seconds(60.0);
        assert_eq!(precisedelta_td(td, "seconds", &[], "%0.2f"), "1 minute");

        let td = TimeDelta::from_seconds(3600.0);
        assert_eq!(precisedelta_td(td, "seconds", &[], "%0.2f"), "1 hour");
    }

    #[test]
    fn test_precisedelta_suppress() {
        let td = TimeDelta::from_days_seconds_micros(2, 3633, 123000);
        assert_eq!(
            precisedelta_td(td, "seconds", &["days"], "%0.2f"),
            "49 hours and 33.12 seconds"
        );
    }

    #[test]
    fn test_rounding_by_fmt() {
        assert!((rounding_by_fmt("%.2f", 1.011) - 1.01).abs() < 1e-9);
        assert!((rounding_by_fmt("%.0f", 1.5) - 2.0).abs() < 1e-9);
        assert_eq!(rounding_by_fmt("%d", 1.999999999999999) as i64, 1);
    }

    #[test]
    fn test_microseconds() {
        let td = TimeDelta::from_days_seconds_micros(0, 0, 1);
        assert_eq!(
            naturaldelta_td(td, true, "microseconds"),
            "1 microsecond"
        );

        let td = TimeDelta::from_days_seconds_micros(0, 0, 4);
        assert_eq!(
            naturaldelta_td(td, true, "microseconds"),
            "4 microseconds"
        );
    }

    #[test]
    fn test_milliseconds() {
        let td = TimeDelta::from_days_seconds_micros(0, 0, 1000);
        assert_eq!(
            naturaldelta_td(td, true, "milliseconds"),
            "1 millisecond"
        );

        let td = TimeDelta::from_days_seconds_micros(0, 0, 4000);
        assert_eq!(
            naturaldelta_td(td, true, "milliseconds"),
            "4 milliseconds"
        );
    }
}
