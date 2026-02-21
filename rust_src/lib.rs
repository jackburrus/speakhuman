use chrono::NaiveDate;
use pyo3::prelude::*;
use pyo3::types::{PyDelta, PyDeltaAccess, PyList};

// ---------------------------------------------------------------------------
// Helper: extract total_seconds from a Python timedelta or float
// ---------------------------------------------------------------------------
fn extract_timedelta_or_float(obj: &Bound<'_, PyAny>) -> PyResult<speakhuman::time::TimeDelta> {
    // Try timedelta first
    if let Ok(delta) = obj.downcast::<PyDelta>() {
        let days = delta.get_days() as i64;
        let seconds = delta.get_seconds() as i64;
        let microseconds = delta.get_microseconds() as i64;
        return Ok(speakhuman::time::TimeDelta::from_days_seconds_micros(
            days,
            seconds,
            microseconds,
        ));
    }
    // Fall back to numeric (float/int)
    let secs: f64 = obj.extract()?;
    Ok(speakhuman::time::TimeDelta::from_seconds(secs))
}

// ---------------------------------------------------------------------------
// Helper: extract a NaiveDate from a Python date/datetime
// ---------------------------------------------------------------------------
fn extract_date(obj: &Bound<'_, PyAny>) -> PyResult<NaiveDate> {
    // Access .year, .month, .day attributes (works for date and datetime)
    let year: i32 = obj.getattr("year")?.extract()?;
    let month: u32 = obj.getattr("month")?.extract()?;
    let day: u32 = obj.getattr("day")?.extract()?;
    NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid date: {}-{}-{}",
            year, month, day
        ))
    })
}

// ===========================================================================
// Lists
// ===========================================================================

/// Convert a list of items into a human-readable string with commas and 'and'.
#[pyfunction]
fn natural_list(items: &Bound<'_, PyList>) -> PyResult<String> {
    let strs: Vec<String> = items
        .iter()
        .map(|item| item.str().map(|s| s.to_string()))
        .collect::<PyResult<_>>()?;
    let refs: Vec<&str> = strs.iter().map(|s| s.as_str()).collect();
    Ok(speakhuman::natural_list(&refs))
}

// ===========================================================================
// Filesize
// ===========================================================================

/// Format a number of bytes like a human-readable filesize (e.g. 10 kB).
#[pyfunction]
#[pyo3(signature = (value, binary=false, gnu=false, format="%.1f"))]
fn naturalsize(
    value: &Bound<'_, PyAny>,
    binary: bool,
    gnu: bool,
    format: &str,
) -> PyResult<String> {
    let bytes: f64 = value.extract().or_else(|_| {
        let s: String = value.extract()?;
        s.parse::<f64>()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    })?;
    Ok(speakhuman::naturalsize(bytes, binary, gnu, format))
}

// ===========================================================================
// Number (non-i18n)
// ===========================================================================

/// Return number in string scientific notation z.wq x 10ⁿ.
#[pyfunction]
#[pyo3(signature = (value, precision=2))]
fn scientific(value: &Bound<'_, PyAny>, precision: usize) -> PyResult<String> {
    let s = value.str()?.to_string();
    Ok(speakhuman::scientific(&s, precision))
}

/// Convert to fractional number.
#[pyfunction]
fn fractional(value: &Bound<'_, PyAny>) -> PyResult<String> {
    let s = value.str()?.to_string();
    Ok(speakhuman::fractional(&s))
}

/// Return a value with a metric SI unit-prefix appended.
#[pyfunction]
#[pyo3(signature = (value, unit="", precision=3))]
fn metric(value: f64, unit: &str, precision: usize) -> PyResult<String> {
    Ok(speakhuman::metric(value, unit, precision))
}

// ===========================================================================
// Number (i18n-aware — English only, fall back to Python for other locales)
// ===========================================================================

/// Converts an integer to its ordinal as a string.
#[pyfunction]
#[pyo3(signature = (value, gender="male"))]
fn ordinal(value: &Bound<'_, PyAny>, gender: &str) -> PyResult<String> {
    let s = value.str()?.to_string();
    Ok(speakhuman::number::ordinal_gendered(&s, gender))
}

/// Converts an integer to a string containing commas every three digits.
#[pyfunction]
#[pyo3(signature = (value, ndigits=None))]
fn intcomma(value: &Bound<'_, PyAny>, ndigits: Option<usize>) -> PyResult<String> {
    let s = value.str()?.to_string();
    Ok(speakhuman::intcomma(&s, ndigits))
}

/// Converts a large integer to a friendly text representation.
#[pyfunction]
#[pyo3(signature = (value, format="%.1f"))]
fn intword(value: &Bound<'_, PyAny>, format: &str) -> PyResult<String> {
    let s = value.str()?.to_string();
    Ok(speakhuman::intword(&s, format))
}

/// Converts an integer to Associated Press style.
#[pyfunction]
fn apnumber(value: &Bound<'_, PyAny>) -> PyResult<String> {
    let s = value.str()?.to_string();
    Ok(speakhuman::apnumber(&s))
}

// ===========================================================================
// Time
// ===========================================================================

/// Return a natural representation of a timedelta or number of seconds.
#[pyfunction]
#[pyo3(signature = (value, months=true, minimum_unit="seconds"))]
fn naturaldelta(
    value: &Bound<'_, PyAny>,
    months: bool,
    minimum_unit: &str,
) -> PyResult<String> {
    let td = extract_timedelta_or_float(value)?;
    Ok(speakhuman::time::naturaldelta_td(td, months, minimum_unit))
}

/// Return a natural day.
#[pyfunction]
#[pyo3(signature = (value, format="%b %d"))]
fn naturalday(value: &Bound<'_, PyAny>, format: &str) -> PyResult<String> {
    match extract_date(value) {
        Ok(date) => Ok(speakhuman::naturalday(date, format)),
        Err(_) => {
            // If we can't extract a date, return str(value) like the Python version
            Ok(value.str()?.to_string())
        }
    }
}

/// Like naturalday, but append a year for dates more than ~five months away.
#[pyfunction]
fn naturaldate(value: &Bound<'_, PyAny>) -> PyResult<String> {
    match extract_date(value) {
        Ok(date) => Ok(speakhuman::naturaldate(date)),
        Err(_) => Ok(value.str()?.to_string()),
    }
}

/// Return a precise representation of a timedelta or number of seconds.
#[pyfunction]
#[pyo3(signature = (value, minimum_unit="seconds", suppress=Vec::new(), format="%0.2f"))]
fn precisedelta(
    value: &Bound<'_, PyAny>,
    minimum_unit: &str,
    suppress: Vec<String>,
    format: &str,
) -> PyResult<String> {
    let td = match extract_timedelta_or_float(value) {
        Ok(td) => td,
        Err(_) => return Ok(value.str()?.to_string()),
    };
    let suppress_refs: Vec<&str> = suppress.iter().map(|s| s.as_str()).collect();
    Ok(speakhuman::time::precisedelta_td(
        td,
        minimum_unit,
        &suppress_refs,
        format,
    ))
}

// ===========================================================================
// Module definition
// ===========================================================================

/// Native Rust acceleration for speakhuman.
#[pymodule]
fn _speakhuman_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Lists
    m.add_function(wrap_pyfunction!(natural_list, m)?)?;
    // Filesize
    m.add_function(wrap_pyfunction!(naturalsize, m)?)?;
    // Number (non-i18n)
    m.add_function(wrap_pyfunction!(scientific, m)?)?;
    m.add_function(wrap_pyfunction!(fractional, m)?)?;
    m.add_function(wrap_pyfunction!(metric, m)?)?;
    // Number (i18n-aware)
    m.add_function(wrap_pyfunction!(ordinal, m)?)?;
    m.add_function(wrap_pyfunction!(intcomma, m)?)?;
    m.add_function(wrap_pyfunction!(intword, m)?)?;
    m.add_function(wrap_pyfunction!(apnumber, m)?)?;
    // Time
    m.add_function(wrap_pyfunction!(naturaldelta, m)?)?;
    m.add_function(wrap_pyfunction!(naturalday, m)?)?;
    m.add_function(wrap_pyfunction!(naturaldate, m)?)?;
    m.add_function(wrap_pyfunction!(precisedelta, m)?)?;
    Ok(())
}
