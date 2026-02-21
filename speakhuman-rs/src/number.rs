//! Humanizing functions for numbers.

use crate::i18n;
use once_cell::sync::Lazy;
use regex::Regex;

static THOUSANDS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(-?\d+)(\d{3})").unwrap());


const HUMAN_POWERS_SINGULAR: &[&str] = &[
    "thousand",
    "million",
    "billion",
    "trillion",
    "quadrillion",
    "quintillion",
    "sextillion",
    "septillion",
    "octillion",
    "nonillion",
    "decillion",
    "googol",
];

const HUMAN_POWERS_PLURAL: &[&str] = &[
    "thousand",
    "million",
    "billion",
    "trillion",
    "quadrillion",
    "quintillion",
    "sextillion",
    "septillion",
    "octillion",
    "nonillion",
    "decillion",
    "googol",
];

/// Handle non-finite float values.
fn format_not_finite(value: f64) -> Option<String> {
    if value.is_nan() {
        Some("NaN".to_string())
    } else if value.is_infinite() && value < 0.0 {
        Some("-Inf".to_string())
    } else if value.is_infinite() && value > 0.0 {
        Some("+Inf".to_string())
    } else {
        None
    }
}

/// Printf-style format for a single float value.
/// Supports: "%.Nf", "%0.Nf", "%N.Mf", "%d", "%i".
pub(crate) fn printf_format(fmt: &str, value: f64) -> String {
    // Handle %d and %i (truncation to int)
    if fmt.contains('d') || fmt.contains('i') {
        return format!("{}", value as i64);
    }
    if let Some(dot_pos) = fmt.find('.') {
        let after_dot = &fmt[dot_pos + 1..];
        if let Some(f_pos) = after_dot.find('f') {
            if let Ok(precision) = after_dot[..f_pos].parse::<usize>() {
                return format!("{:.prec$}", value, prec = precision);
            }
        }
    }
    format!("{}", value)
}

/// Convert a float to a fraction with limited denominator, similar to
/// Python's `Fraction(f).limit_denominator(max_denom)`.
fn float_to_fraction(value: f64, max_denom: i64) -> (i64, i64) {
    if value == 0.0 {
        return (0, 1);
    }

    let negative = value < 0.0;
    let value = value.abs();

    // Use continued fraction algorithm (Stern-Brocot tree)
    // to find best rational approximation with denominator <= max_denom.
    let mut p0: i64 = 0;
    let mut q0: i64 = 1;
    let mut p1: i64 = 1;
    let mut q1: i64 = 0;
    let mut x = value;

    loop {
        let a = x.floor() as i64;
        let p2 = a * p1 + p0;
        let q2 = a * q1 + q0;

        if q2 > max_denom {
            break;
        }

        p0 = p1;
        q0 = q1;
        p1 = p2;
        q1 = q2;

        let remainder = x - a as f64;
        if remainder.abs() < 1e-10 {
            break;
        }
        x = 1.0 / remainder;

        if x > 1e10 {
            break;
        }
    }

    // Also check the last convergent that fits
    if q1 == 0 {
        return (0, 1);
    }

    let numer = if negative { -p1 } else { p1 };
    (numer, q1)
}

/// Converts an integer to its ordinal as a string.
///
/// For example, 1 is "1st", 2 is "2nd", 3 is "3rd", etc.
///
/// # Examples
/// ```
/// use speakhuman::number::ordinal;
/// assert_eq!(ordinal("1"), "1st");
/// assert_eq!(ordinal("2"), "2nd");
/// assert_eq!(ordinal("3"), "3rd");
/// assert_eq!(ordinal("4"), "4th");
/// assert_eq!(ordinal("11"), "11th");
/// assert_eq!(ordinal("12"), "12th");
/// assert_eq!(ordinal("13"), "13th");
/// assert_eq!(ordinal("101"), "101st");
/// assert_eq!(ordinal("something else"), "something else");
/// ```
pub fn ordinal(value: &str) -> String {
    ordinal_gendered(value, "male")
}

/// Converts an integer to its ordinal with gender support.
pub fn ordinal_gendered(value: &str, gender: &str) -> String {
    // Try to parse as float first to check for non-finite
    if let Ok(f) = value.parse::<f64>() {
        if !f.is_finite() {
            return format_not_finite(f).unwrap();
        }
    }

    // Try to parse as integer
    let int_val: i64 = match value.parse::<f64>() {
        Ok(f) => f as i64,
        Err(_) => return value.to_string(),
    };

    let suffixes = if gender == "male" {
        [
            i18n::pgettext("0 (male)", "th"),
            i18n::pgettext("1 (male)", "st"),
            i18n::pgettext("2 (male)", "nd"),
            i18n::pgettext("3 (male)", "rd"),
            i18n::pgettext("4 (male)", "th"),
            i18n::pgettext("5 (male)", "th"),
            i18n::pgettext("6 (male)", "th"),
            i18n::pgettext("7 (male)", "th"),
            i18n::pgettext("8 (male)", "th"),
            i18n::pgettext("9 (male)", "th"),
        ]
    } else {
        [
            i18n::pgettext("0 (female)", "th"),
            i18n::pgettext("1 (female)", "st"),
            i18n::pgettext("2 (female)", "nd"),
            i18n::pgettext("3 (female)", "rd"),
            i18n::pgettext("4 (female)", "th"),
            i18n::pgettext("5 (female)", "th"),
            i18n::pgettext("6 (female)", "th"),
            i18n::pgettext("7 (female)", "th"),
            i18n::pgettext("8 (female)", "th"),
            i18n::pgettext("9 (female)", "th"),
        ]
    };

    let abs_val = int_val.unsigned_abs();
    if abs_val % 100 == 11 || abs_val % 100 == 12 || abs_val % 100 == 13 {
        format!("{}{}", int_val, suffixes[0])
    } else {
        format!("{}{}", int_val, suffixes[(abs_val % 10) as usize])
    }
}

/// Converts an integer to a string containing commas every three digits.
///
/// # Examples
/// ```
/// use speakhuman::number::intcomma;
/// assert_eq!(intcomma("100", None), "100");
/// assert_eq!(intcomma("1000", None), "1,000");
/// assert_eq!(intcomma("1000000", None), "1,000,000");
/// ```
pub fn intcomma(value: &str, ndigits: Option<usize>) -> String {
    let thousands_sep = i18n::thousands_separator();
    let decimal_sep = i18n::decimal_separator();

    // Clean input: remove existing separators
    let cleaned = value
        .replace(&thousands_sep, "")
        .replace(&decimal_sep, ".");

    // Try to parse as float to check for non-finite
    match cleaned.parse::<f64>() {
        Ok(f) if !f.is_finite() => return format_not_finite(f).unwrap(),
        Err(_) => return value.to_string(),
        _ => {}
    }

    let orig = if let Some(nd) = ndigits {
        let f: f64 = cleaned.parse().unwrap_or(0.0);
        format!("{:.prec$}", f, prec = nd)
    } else if cleaned.contains('.') {
        // Preserve original decimal representation
        let f: f64 = cleaned.parse().unwrap_or(0.0);
        // Use the parsed float to check, but keep original string representation
        // to preserve trailing digits like Python does
        if cleaned.parse::<i64>().is_ok() {
            cleaned.clone()
        } else {
            // For float strings, we need to handle precision carefully
            // Parse as float and format to preserve Python's behavior
            let s = format!("{}", f);
            // Use whichever has more decimal precision
            if cleaned.len() > s.len() {
                cleaned.clone()
            } else {
                s
            }
        }
    } else {
        match cleaned.parse::<i64>() {
            Ok(i) => i.to_string(),
            Err(_) => {
                match cleaned.parse::<f64>() {
                    Ok(f) => format!("{}", f),
                    Err(_) => return value.to_string(),
                }
            }
        }
    };

    // Replace decimal point with locale-specific separator
    let orig = orig.replace('.', &decimal_sep);

    // Insert thousands separators using regex (compiled once)
    let mut result = orig;
    loop {
        let new = THOUSANDS_RE
            .replace(&result, |caps: &regex::Captures| {
                format!("{}{}{}", &caps[1], thousands_sep, &caps[2])
            })
            .to_string();
        if new == result {
            return result;
        }
        result = new;
    }
}

/// Converts a large integer to a friendly text representation.
///
/// Works best for numbers over 1 million.
///
/// # Examples
/// ```
/// use speakhuman::number::intword;
/// assert_eq!(intword("100", "%.1f"), "100");
/// assert_eq!(intword("1000000", "%.1f"), "1.0 million");
/// assert_eq!(intword("1200000000", "%.1f"), "1.2 billion");
/// ```
pub fn intword(value: &str, format: &str) -> String {
    // Parse as f64, working directly with floats to avoid i128 overflow for
    // values > 1.7e38 (like googol = 10^100)
    let f_val: f64 = match value.replace('_', "").parse::<f64>() {
        Ok(f) => f,
        Err(_) => return value.to_string(),
    };

    if !f_val.is_finite() {
        return format_not_finite(f_val).unwrap();
    }

    let negative = f_val < 0.0;
    let abs_f64 = f_val.abs();
    let negative_prefix = if negative { "-" } else { "" };

    if abs_f64 < 1000.0 {
        // Display as integer for small values
        return format!("{}{}", negative_prefix, abs_f64 as i64);
    }

    // Use f64 powers to avoid u128 overflow for googol (10^100)
    let powers_f64: &[f64] = &[
        1e3, 1e6, 1e9, 1e12, 1e15, 1e18, 1e21, 1e24, 1e27, 1e30, 1e33, 1e100,
    ];

    // Find the right power
    let ordinal = match powers_f64
        .iter()
        .position(|&p| p > abs_f64)
    {
        Some(0) => return format!("{}{}", negative_prefix, abs_f64 as i64),
        Some(i) => i - 1,
        None => powers_f64.len() - 1,
    };

    let largest_ordinal = ordinal == powers_f64.len() - 1;
    let power = powers_f64[ordinal];
    let chopped = abs_f64 / power;
    let rounded_value = printf_format(format, chopped)
        .parse::<f64>()
        .unwrap_or(chopped);

    let (final_ordinal, final_value) = if !largest_ordinal
        && ordinal + 1 < powers_f64.len()
        && (rounded_value * power - powers_f64[ordinal + 1]).abs() < 1.0
    {
        (ordinal + 1, 1.0)
    } else {
        (ordinal, rounded_value)
    };

    let singular = HUMAN_POWERS_SINGULAR[final_ordinal];
    let plural = HUMAN_POWERS_PLURAL[final_ordinal];
    let unit = i18n::ngettext(singular, plural, final_value.ceil() as i64);
    let decimal_sep = i18n::decimal_separator();
    let number = printf_format(format, final_value).replace('.', &decimal_sep);
    format!("{}{} {}", negative_prefix, number, unit)
}

/// Converts an integer to Associated Press style.
///
/// For numbers 0-9, returns the word. Otherwise returns the number as string.
///
/// # Examples
/// ```
/// use speakhuman::number::apnumber;
/// assert_eq!(apnumber("0"), "zero");
/// assert_eq!(apnumber("5"), "five");
/// assert_eq!(apnumber("10"), "10");
/// ```
pub fn apnumber(value: &str) -> String {
    if let Ok(f) = value.parse::<f64>() {
        if !f.is_finite() {
            return format_not_finite(f).unwrap();
        }
    }

    let int_val: i64 = match value.parse::<f64>() {
        Ok(f) => f as i64,
        Err(_) => return value.to_string(),
    };

    if !(0..10).contains(&int_val) {
        return int_val.to_string();
    }

    let words = [
        i18n::gettext("zero"),
        i18n::gettext("one"),
        i18n::gettext("two"),
        i18n::gettext("three"),
        i18n::gettext("four"),
        i18n::gettext("five"),
        i18n::gettext("six"),
        i18n::gettext("seven"),
        i18n::gettext("eight"),
        i18n::gettext("nine"),
    ];
    words[int_val as usize].clone()
}

/// Convert to fractional number.
///
/// # Examples
/// ```
/// use speakhuman::number::fractional;
/// assert_eq!(fractional("0.3"), "3/10");
/// assert_eq!(fractional("1.3"), "1 3/10");
/// assert_eq!(fractional("1"), "1");
/// ```
pub fn fractional(value: &str) -> String {
    let number: f64 = match value.parse() {
        Ok(f) => f,
        Err(_) => return value.to_string(),
    };

    if !number.is_finite() {
        return format_not_finite(number).unwrap();
    }

    let whole_number = number as i64;
    let frac_part = number - whole_number as f64;

    // Implement limit_denominator similar to Python's Fraction.limit_denominator(1000)
    let (numerator, denominator) = float_to_fraction(frac_part, 1000);

    if whole_number != 0 && numerator == 0 && denominator == 1 {
        return format!("{}", whole_number);
    }

    if whole_number == 0 {
        return format!("{}/{}", numerator, denominator);
    }

    format!("{} {}/{}", whole_number, numerator, denominator)
}

/// Return number in string scientific notation z.wq x 10^n.
///
/// Uses Unicode superscript characters for the exponent.
///
/// # Examples
/// ```
/// use speakhuman::number::scientific;
/// assert_eq!(scientific("1000", 2), "1.00 x 10³");
/// assert_eq!(scientific("0.3", 2), "3.00 x 10⁻¹");
/// ```
pub fn scientific(value: &str, precision: usize) -> String {
    let f: f64 = match value.parse() {
        Ok(v) => v,
        Err(_) => return value.to_string(),
    };

    if !f.is_finite() {
        return format_not_finite(f).unwrap();
    }

    let formatted = format!("{:.prec$e}", f, prec = precision);

    // Rust formats as "1.00e2" or "1.00e-2", we need to split on 'e'
    let parts: Vec<&str> = formatted.split('e').collect();
    if parts.len() != 2 {
        return formatted;
    }

    let mantissa = parts[0];
    let mut exp_str = parts[1].to_string();

    // Remove leading '+' and leading zeros (but keep at least one digit)
    if exp_str.starts_with('+') {
        exp_str = exp_str[1..].to_string();
    }
    // Remove leading zeros but keep the sign
    let (sign, digits) = if exp_str.starts_with('-') {
        ("-", exp_str[1..].trim_start_matches('0'))
    } else {
        ("", exp_str.trim_start_matches('0'))
    };
    let digits = if digits.is_empty() { "0" } else { digits };
    let exp_clean = format!("{}{}", sign, digits);

    let exponent_map: std::collections::HashMap<char, char> = [
        ('0', '\u{2070}'),
        ('1', '\u{00B9}'),
        ('2', '\u{00B2}'),
        ('3', '\u{00B3}'),
        ('4', '\u{2074}'),
        ('5', '\u{2075}'),
        ('6', '\u{2076}'),
        ('7', '\u{2077}'),
        ('8', '\u{2078}'),
        ('9', '\u{2079}'),
        ('-', '\u{207B}'),
    ]
    .iter()
    .cloned()
    .collect();

    let superscript: String = exp_clean
        .chars()
        .filter_map(|c| exponent_map.get(&c).copied())
        .collect();

    format!("{} x 10{}", mantissa, superscript)
}

/// Possible format types for clamp.
pub enum ClampFormat {
    Str(String),
    Fn(Box<dyn Fn(f64) -> String>),
}

/// Returns number with the specified format, clamped between floor and ceil.
///
/// # Examples
/// ```
/// use speakhuman::number::{clamp, ClampFormat};
/// assert_eq!(clamp(123.456, &ClampFormat::Str("{:}".to_string()), None, None, "<", ">"), Some("123.456".to_string()));
/// ```
pub fn clamp(
    value: f64,
    format: &ClampFormat,
    floor: Option<f64>,
    ceil: Option<f64>,
    floor_token: &str,
    ceil_token: &str,
) -> Option<String> {
    if value.is_nan() {
        return Some("NaN".to_string());
    }
    if value.is_infinite() {
        return Some(format_not_finite(value).unwrap());
    }

    let (clamped, token) = if let Some(f) = floor {
        if value < f {
            (f, floor_token)
        } else if let Some(c) = ceil {
            if value > c {
                (c, ceil_token)
            } else {
                (value, "")
            }
        } else {
            (value, "")
        }
    } else if let Some(c) = ceil {
        if value > c {
            (c, ceil_token)
        } else {
            (value, "")
        }
    } else {
        (value, "")
    };

    let formatted = match format {
        ClampFormat::Str(fmt) => {
            // Handle Rust-style format strings like "{:}", "{:.0%}", etc.
            if fmt.contains('%') {
                // Percentage format: "{:.0%}" means multiply by 100 and add %
                let precision = if let Some(start) = fmt.find('.') {
                    let after = &fmt[start + 1..];
                    after
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<usize>()
                        .unwrap_or(0)
                } else {
                    0
                };
                format!("{:.prec$}%", clamped * 100.0, prec = precision)
            } else if fmt == "{:}" {
                format!("{}", clamped)
            } else {
                format!("{}", clamped)
            }
        }
        ClampFormat::Fn(f) => f(clamped),
    };

    Some(format!("{}{}", token, formatted))
}

/// Return a value with a metric SI unit-prefix appended.
///
/// # Examples
/// ```
/// use speakhuman::number::metric;
/// assert_eq!(metric(1500.0, "V", 3), "1.50 kV");
/// assert_eq!(metric(200_000.0, "", 3), "200 k");
/// ```
pub fn metric(value: f64, unit: &str, precision: usize) -> String {
    if !value.is_finite() {
        return format_not_finite(value).unwrap();
    }

    let exponent = if value != 0.0 {
        value.abs().log10().floor() as i32
    } else {
        0
    };

    if exponent >= 33 || exponent < -30 {
        let s = scientific(&value.to_string(), precision.saturating_sub(1));
        return format!("{}{}", s, unit);
    }

    // Python uses floor division for negative numbers: -4 // 3 = -2, not -1
    let exp_div_3 = if exponent >= 0 {
        exponent / 3
    } else {
        (exponent - 2) / 3 // floor division for negatives
    };
    let scaled = value / 10f64.powi(exp_div_3 * 3);

    let ordinal = if exponent >= 3 {
        let idx = (exponent / 3 - 1) as usize;
        let prefixes = ['k', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y', 'R', 'Q'];
        if idx < prefixes.len() {
            prefixes[idx].to_string()
        } else {
            String::new()
        }
    } else if exponent < 0 {
        let idx = ((-exponent - 1) / 3) as usize;
        let prefixes = ['m', '\u{03BC}', 'n', 'p', 'f', 'a', 'z', 'y', 'r', 'q'];
        if idx < prefixes.len() {
            prefixes[idx].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let exp_mod_3 = ((exponent % 3) + 3) % 3; // Python-style modulo (always non-negative)
    let prec = precision as i32 - exp_mod_3 - 1;
    let prec = prec.max(0) as usize;
    let formatted = format!("{:.prec$}", scaled, prec = prec);

    let space = if (!unit.is_empty() || !ordinal.is_empty())
        && unit != "°" && unit != "′" && unit != "″"
    {
        " "
    } else {
        ""
    };

    format!("{}{}{}{}", formatted, space, ordinal, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordinal() {
        assert_eq!(ordinal("1"), "1st");
        assert_eq!(ordinal("2"), "2nd");
        assert_eq!(ordinal("3"), "3rd");
        assert_eq!(ordinal("4"), "4th");
        assert_eq!(ordinal("11"), "11th");
        assert_eq!(ordinal("12"), "12th");
        assert_eq!(ordinal("13"), "13th");
        assert_eq!(ordinal("101"), "101st");
        assert_eq!(ordinal("102"), "102nd");
        assert_eq!(ordinal("103"), "103rd");
        assert_eq!(ordinal("111"), "111th");
        assert_eq!(ordinal("something else"), "something else");
        assert_eq!(ordinal("nan"), "NaN");
        assert_eq!(ordinal("-inf"), "-Inf");
    }

    #[test]
    fn test_intcomma() {
        assert_eq!(intcomma("100", None), "100");
        assert_eq!(intcomma("1000", None), "1,000");
        assert_eq!(intcomma("10123", None), "10,123");
        assert_eq!(intcomma("1000000", None), "1,000,000");
        assert_eq!(intcomma("nan", None), "NaN");
        assert_eq!(intcomma("-inf", None), "-Inf");
    }

    #[test]
    fn test_intcomma_with_precision() {
        assert_eq!(intcomma("1234567.1234567", Some(0)), "1,234,567");
        assert_eq!(intcomma("1234567.1234567", Some(1)), "1,234,567.1");
        assert_eq!(intcomma("1234567", Some(1)), "1,234,567.0");
    }

    #[test]
    fn test_intword() {
        assert_eq!(intword("100", "%.1f"), "100");
        assert_eq!(intword("1000", "%.1f"), "1.0 thousand");
        assert_eq!(intword("12400", "%.1f"), "12.4 thousand");
        assert_eq!(intword("1000000", "%.1f"), "1.0 million");
        assert_eq!(intword("-1000000", "%.1f"), "-1.0 million");
        assert_eq!(intword("1200000000", "%.1f"), "1.2 billion");
        assert_eq!(intword("nan", "%.1f"), "NaN");
        assert_eq!(intword("-inf", "%.1f"), "-Inf");
    }

    #[test]
    fn test_intword_format() {
        assert_eq!(intword("1230000", "%0.2f"), "1.23 million");
        assert_eq!(intword("1234567", "%.0f"), "1 million");
        assert_eq!(intword("999500", "%.0f"), "1 million");
        assert_eq!(intword("999499", "%.0f"), "999 thousand");
    }

    #[test]
    fn test_apnumber() {
        assert_eq!(apnumber("0"), "zero");
        assert_eq!(apnumber("1"), "one");
        assert_eq!(apnumber("5"), "five");
        assert_eq!(apnumber("9"), "nine");
        assert_eq!(apnumber("10"), "10");
        assert_eq!(apnumber("foo"), "foo");
        assert_eq!(apnumber("nan"), "NaN");
    }

    #[test]
    fn test_fractional() {
        assert_eq!(fractional("1"), "1");
        assert_eq!(fractional("0.3"), "3/10");
        assert_eq!(fractional("1.5"), "1 1/2");
        assert_eq!(fractional("ten"), "ten");
        assert_eq!(fractional("nan"), "NaN");
        assert_eq!(fractional("inf"), "+Inf");
        assert_eq!(fractional("-inf"), "-Inf");
    }

    #[test]
    fn test_scientific() {
        assert_eq!(scientific("1000", 2), "1.00 x 10\u{00B3}");
        assert_eq!(scientific("-1000", 2), "-1.00 x 10\u{00B3}");
        assert_eq!(scientific("5.5", 2), "5.50 x 10\u{2070}");
        assert_eq!(scientific("0.3", 2), "3.00 x 10\u{207B}\u{00B9}");
        assert_eq!(scientific("foo", 2), "foo");
        assert_eq!(scientific("nan", 2), "NaN");
    }

    #[test]
    fn test_metric() {
        assert_eq!(metric(1500.0, "V", 3), "1.50 kV");
        assert_eq!(metric(2e8, "W", 3), "200 MW");
        assert_eq!(metric(220e-6, "F", 3), "220 \u{03BC}F");
        assert_eq!(metric(0.0, "", 3), "0.00");
    }

    #[test]
    fn test_metric_no_space_for_degree() {
        assert_eq!(metric(1.0, "°", 3), "1.00°");
    }
}
