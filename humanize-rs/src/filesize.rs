//! Bits and bytes related humanization.

const SUFFIXES_DECIMAL: &[&str] = &[
    " kB", " MB", " GB", " TB", " PB", " EB", " ZB", " YB", " RB", " QB",
];
const SUFFIXES_BINARY: &[&str] = &[
    " KiB", " MiB", " GiB", " TiB", " PiB", " EiB", " ZiB", " YiB", " RiB", " QiB",
];
const SUFFIXES_GNU: &[&str] = &["K", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q"];

/// Format a number of bytes like a human-readable filesize (e.g. 10 kB).
///
/// By default, decimal suffixes (kB, MB) are used.
///
/// # Examples
/// ```
/// use humanize::filesize::naturalsize;
/// assert_eq!(naturalsize(3_000_000.0, false, false, "%.1f"), "3.0 MB");
/// assert_eq!(naturalsize(300.0, false, true, "%.1f"), "300B");
/// assert_eq!(naturalsize(3000.0, true, false, "%.1f"), "2.9 KiB");
/// ```
pub fn naturalsize(value: f64, binary: bool, gnu: bool, format: &str) -> String {
    let suffix: &[&str] = if gnu {
        SUFFIXES_GNU
    } else if binary {
        SUFFIXES_BINARY
    } else {
        SUFFIXES_DECIMAL
    };

    let base: f64 = if gnu || binary { 1024.0 } else { 1000.0 };
    let abs_bytes = value.abs();

    if abs_bytes == 1.0 && !gnu {
        return format!("{} Byte", value as i64);
    }

    if abs_bytes < base {
        return if gnu {
            format!("{}B", value as i64)
        } else {
            format!("{} Bytes", value as i64)
        };
    }

    let exp = (abs_bytes.log(base) as usize).min(suffix.len());
    let divided = value / base.powi(exp as i32);
    let formatted = printf_format(format, divided);
    format!("{}{}", formatted, suffix[exp - 1])
}

/// Simple printf-style format for a single float value.
/// Supports formats like "%.1f", "%.2f", "%.3f", "%.0f", "%0.2f".
fn printf_format(fmt: &str, value: f64) -> String {
    // Parse the format string to extract precision
    if let Some(dot_pos) = fmt.find('.') {
        let after_dot = &fmt[dot_pos + 1..];
        if let Some(f_pos) = after_dot.find('f') {
            if let Ok(precision) = after_dot[..f_pos].parse::<usize>() {
                return format!("{:.prec$}", value, prec = precision);
            }
        }
    }
    // Fallback: default formatting
    format!("{}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naturalsize_decimal() {
        assert_eq!(naturalsize(300.0, false, false, "%.1f"), "300 Bytes");
        assert_eq!(naturalsize(1000.0, false, false, "%.1f"), "1.0 kB");
        assert_eq!(naturalsize(1e6, false, false, "%.1f"), "1.0 MB");
        assert_eq!(naturalsize(1e9, false, false, "%.1f"), "1.0 GB");
        assert_eq!(naturalsize(1e12, false, false, "%.1f"), "1.0 TB");
        assert_eq!(naturalsize(1e15, false, false, "%.1f"), "1.0 PB");
        assert_eq!(naturalsize(1e18, false, false, "%.1f"), "1.0 EB");
        assert_eq!(naturalsize(1e21, false, false, "%.1f"), "1.0 ZB");
        assert_eq!(naturalsize(1e24, false, false, "%.1f"), "1.0 YB");
        assert_eq!(naturalsize(1e27, false, false, "%.1f"), "1.0 RB");
        assert_eq!(naturalsize(1e30, false, false, "%.1f"), "1.0 QB");
    }

    #[test]
    fn test_naturalsize_binary() {
        assert_eq!(naturalsize(300.0, true, false, "%.1f"), "300 Bytes");
        assert_eq!(
            naturalsize(1024.0 * 31.0, true, false, "%.1f"),
            "31.0 KiB"
        );
        assert_eq!(
            naturalsize(1048576.0 * 32.0, true, false, "%.1f"),
            "32.0 MiB"
        );
    }

    #[test]
    fn test_naturalsize_gnu() {
        assert_eq!(naturalsize(300.0, false, true, "%.1f"), "300B");
        assert_eq!(naturalsize(3000.0, false, true, "%.1f"), "2.9K");
        assert_eq!(naturalsize(3000000.0, false, true, "%.1f"), "2.9M");
        assert_eq!(naturalsize(1024.0, false, true, "%.1f"), "1.0K");
    }

    #[test]
    fn test_naturalsize_single_byte() {
        assert_eq!(naturalsize(1.0, false, false, "%.1f"), "1 Byte");
    }

    #[test]
    fn test_naturalsize_custom_format() {
        assert_eq!(
            naturalsize(3141592.0, false, false, "%.2f"),
            "3.14 MB"
        );
        assert_eq!(naturalsize(3000.0, false, true, "%.3f"), "2.930K");
        assert_eq!(
            naturalsize(3000000000.0, false, true, "%.0f"),
            "3G"
        );
    }

    #[test]
    fn test_naturalsize_negative() {
        assert_eq!(naturalsize(-4096.0, true, false, "%.1f"), "-4.0 KiB");
        assert_eq!(naturalsize(-300.0, false, false, "%.1f"), "-300 Bytes");
    }
}
