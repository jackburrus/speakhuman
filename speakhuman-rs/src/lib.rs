//! Speakhuman - human-readable formatting for numbers, dates, times, and file sizes.
//!
//! This library provides functions to convert machine-readable values into
//! human-readable formats:
//! - Numbers (ordinals, word representation, fractional, scientific notation, SI units)
//! - Dates/Times (natural language time deltas and dates)
//! - File sizes (with binary/decimal/GNU formatting options)
//! - Lists (natural comma-and-and formatting)
//! - Internationalization support (30+ locales via .mo files)

pub mod filesize;
pub mod i18n;
pub mod lists;
pub mod number;
pub mod time;

// Re-exports for convenience
pub use filesize::naturalsize;
pub use i18n::{activate, deactivate, decimal_separator, thousands_separator};
pub use lists::natural_list;
pub use number::{apnumber, clamp, fractional, intcomma, intword, metric, ordinal, scientific};
pub use time::{
    naturaldate, naturalday, naturaldelta, naturaldelta_td, naturaltime_delta, precisedelta,
    precisedelta_td, TimeDelta, Unit,
};
