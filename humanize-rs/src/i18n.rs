//! Internationalization support.
//!
//! Provides activate/deactivate for locale selection, and translation functions
//! that mirror Python's gettext (_gettext, _pgettext, _ngettext).
//! Uses thread-local state so different threads can have different locales.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a loaded set of translations from a .mo file.
#[derive(Clone, Debug)]
pub struct Translations {
    /// msgid -> msgstr mapping (simple translations)
    messages: HashMap<String, String>,
    /// For plural forms: msgid -> (singular_translation, plural_translation)
    plurals: HashMap<String, Vec<String>>,
    /// Number of plural forms
    nplurals: u32,
}

impl Translations {
    fn null() -> Self {
        Self {
            messages: HashMap::new(),
            plurals: HashMap::new(),
            nplurals: 2,
        }
    }

    fn gettext(&self, message: &str) -> String {
        self.messages
            .get(message)
            .cloned()
            .unwrap_or_else(|| message.to_string())
    }

    fn pgettext(&self, context: &str, message: &str) -> String {
        let key = format!("{}\x04{}", context, message);
        self.messages
            .get(&key)
            .cloned()
            .unwrap_or_else(|| message.to_string())
    }

    fn ngettext(&self, singular: &str, plural: &str, n: i64) -> String {
        if let Some(forms) = self.plurals.get(singular) {
            let idx = self.plural_index(n);
            if idx < forms.len() {
                return forms[idx].clone();
            }
        }
        // Fallback to simple lookup
        if let Some(msg) = self.messages.get(singular) {
            if n == 1 {
                return msg.clone();
            }
            // Check if there's a plural form stored
            if let Some(plural_msg) = self.messages.get(plural) {
                return plural_msg.clone();
            }
            return msg.clone();
        }
        if n == 1 {
            singular.to_string()
        } else {
            plural.to_string()
        }
    }

    fn plural_index(&self, n: i64) -> usize {
        // Default English plural rule: 0 if n==1, else 1
        // For Slavic languages this would be more complex, but gettext handles
        // it via the Plural-Forms header which we parse
        if self.nplurals == 1 {
            0
        } else if self.nplurals == 2 {
            if n == 1 { 0 } else { 1 }
        } else if self.nplurals == 3 {
            // Slavic languages (Russian, Polish, etc.)
            let n_abs = n.unsigned_abs();
            if n_abs % 10 == 1 && n_abs % 100 != 11 {
                0
            } else if n_abs % 10 >= 2 && n_abs % 10 <= 4
                && (n_abs % 100 < 10 || n_abs % 100 >= 20)
            {
                1
            } else {
                2
            }
        } else {
            if n == 1 { 0 } else { 1 }
        }
    }
}

/// Parse a .mo file and return a Translations struct.
fn parse_mo_file(path: &Path) -> Result<Translations, String> {
    let data = fs::read(path).map_err(|e| format!("Cannot read .mo file: {}", e))?;

    if data.len() < 28 {
        return Err("Invalid .mo file: too short".into());
    }

    // Check magic number
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let (read_u32, _is_le): (fn(&[u8], usize) -> u32, bool) =
        if magic == 0x950412de {
            (read_u32_le, true)
        } else if magic == 0xde120495 {
            (read_u32_be, false)
        } else {
            return Err("Invalid .mo file: bad magic number".into());
        };

    let num_strings = read_u32(&data, 8) as usize;
    let orig_offset = read_u32(&data, 12) as usize;
    let trans_offset = read_u32(&data, 16) as usize;

    let mut messages = HashMap::new();
    let mut plurals = HashMap::new();
    let mut nplurals = 2u32;

    for i in 0..num_strings {
        let orig_len = read_u32(&data, orig_offset + i * 8) as usize;
        let orig_start = read_u32(&data, orig_offset + i * 8 + 4) as usize;

        let trans_len = read_u32(&data, trans_offset + i * 8) as usize;
        let trans_start = read_u32(&data, trans_offset + i * 8 + 4) as usize;

        if orig_start + orig_len > data.len() || trans_start + trans_len > data.len() {
            continue;
        }

        let orig = String::from_utf8_lossy(&data[orig_start..orig_start + orig_len]).to_string();
        let trans =
            String::from_utf8_lossy(&data[trans_start..trans_start + trans_len]).to_string();

        if orig.is_empty() {
            // Metadata entry - parse Plural-Forms
            for line in trans.lines() {
                if line.starts_with("Plural-Forms:") {
                    if let Some(np) = line.split("nplurals=").nth(1) {
                        if let Some(num_str) = np.split(';').next() {
                            if let Ok(n) = num_str.trim().parse::<u32>() {
                                nplurals = n;
                            }
                        }
                    }
                }
            }
            continue;
        }

        // Check for plural forms (separated by \0)
        if orig.contains('\0') {
            let orig_parts: Vec<&str> = orig.splitn(2, '\0').collect();
            let trans_parts: Vec<String> =
                trans.split('\0').map(|s| s.to_string()).collect();
            // Store singular key for lookup
            plurals.insert(orig_parts[0].to_string(), trans_parts.clone());
            // Also store the first form in simple messages
            if let Some(first) = trans_parts.first() {
                messages.insert(orig_parts[0].to_string(), first.clone());
            }
            if trans_parts.len() > 1 {
                if let Some(second) = trans_parts.get(1) {
                    messages.insert(orig_parts[1].to_string(), second.clone());
                }
            }
        } else {
            messages.insert(orig, trans);
        }
    }

    Ok(Translations {
        messages,
        plurals,
        nplurals,
    })
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn read_u32_be(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

// Mapping of locale to thousands separator
fn thousands_separator_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("de_DE", ".");
    m.insert("fr_FR", "\u{00a0}"); // non-breaking space
    m.insert("it_IT", ".");
    m.insert("pt_BR", ".");
    m.insert("hu_HU", "\u{00a0}");
    m
}

fn decimal_separator_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("de_DE", ",");
    m.insert("fr_FR", ".");
    m.insert("it_IT", ",");
    m.insert("pt_BR", ",");
    m.insert("hu_HU", ",");
    m
}

/// Thread-local state for the current locale and cached translations.
struct I18nState {
    locale: Option<String>,
    translations: HashMap<Option<String>, Translations>,
}

impl I18nState {
    fn new() -> Self {
        let mut translations = HashMap::new();
        translations.insert(None, Translations::null());
        Self {
            locale: None,
            translations,
        }
    }
}

thread_local! {
    static I18N_STATE: RefCell<I18nState> = RefCell::new(I18nState::new());
}

/// Return the default locale path (relative to the crate, looking for ../locale or similar).
/// In the Rust port, callers should provide the path explicitly. This tries to find
/// the locale directory from the Python source.
fn get_default_locale_path() -> Option<PathBuf> {
    // Try to locate relative to the humanize-rs crate
    let candidates = [
        // When running from humanize-rs directory
        PathBuf::from("../src/humanize/locale"),
        // When running from repo root
        PathBuf::from("src/humanize/locale"),
        // Installed alongside
        PathBuf::from("locale"),
    ];
    for p in &candidates {
        if p.exists() {
            return Some(p.clone());
        }
    }
    None
}

/// Activate a locale for translations.
///
/// # Arguments
/// * `locale` - Language name, e.g. "ru_RU". If None or starts with "en", defaults to no translation.
/// * `path` - Optional path to the locale directory containing .mo files.
pub fn activate(locale: Option<&str>, path: Option<&Path>) -> Result<(), String> {
    let locale = match locale {
        None => {
            deactivate();
            return Ok(());
        }
        Some(l) if l.starts_with("en") => {
            deactivate();
            return Ok(());
        }
        Some(l) => l.to_string(),
    };

    I18N_STATE.with(|state| {
        let mut state = state.borrow_mut();

        if !state.translations.contains_key(&Some(locale.clone())) {
            let locale_path = match path {
                Some(p) => p.to_path_buf(),
                None => get_default_locale_path().ok_or_else(|| {
                    "Humanize cannot determinate the default location of the 'locale' folder. \
                     You need to pass the path explicitly."
                        .to_string()
                })?,
            };

            let mo_path = locale_path
                .join(&locale)
                .join("LC_MESSAGES")
                .join("humanize.mo");

            if !mo_path.exists() {
                // Try with just the language code (e.g., "fr" from "fr_FR")
                let lang_code = locale.split('_').next().unwrap_or(&locale);
                let alt_mo_path = locale_path
                    .join(lang_code)
                    .join("LC_MESSAGES")
                    .join("humanize.mo");
                if alt_mo_path.exists() {
                    let translations = parse_mo_file(&alt_mo_path)?;
                    state
                        .translations
                        .insert(Some(locale.clone()), translations);
                } else {
                    return Err(format!(
                        "Cannot find .mo file at {:?} or {:?}",
                        mo_path, alt_mo_path
                    ));
                }
            } else {
                let translations = parse_mo_file(&mo_path)?;
                state
                    .translations
                    .insert(Some(locale.clone()), translations);
            }
        }

        state.locale = Some(locale);
        Ok(())
    })
}

/// Deactivate internationalization (revert to English/no translation).
pub fn deactivate() {
    I18N_STATE.with(|state| {
        state.borrow_mut().locale = None;
    });
}

/// Get the current translations.
fn get_translation() -> Translations {
    I18N_STATE.with(|state| {
        let state = state.borrow();
        state
            .translations
            .get(&state.locale)
            .cloned()
            .unwrap_or_else(Translations::null)
    })
}

/// Translate a message using the current locale.
pub fn gettext(message: &str) -> String {
    get_translation().gettext(message)
}

/// Translate a message with context using the current locale.
pub fn pgettext(context: &str, message: &str) -> String {
    get_translation().pgettext(context, message)
}

/// Translate with plural forms.
pub fn ngettext(singular: &str, plural: &str, n: i64) -> String {
    get_translation().ngettext(singular, plural, n)
}

/// Return the thousands separator for the current locale (default: ",").
pub fn thousands_separator() -> String {
    I18N_STATE.with(|state| {
        let state = state.borrow();
        let map = thousands_separator_map();
        match &state.locale {
            Some(locale) => map.get(locale.as_str()).unwrap_or(&",").to_string(),
            None => ",".to_string(),
        }
    })
}

/// Return the decimal separator for the current locale (default: ".").
pub fn decimal_separator() -> String {
    I18N_STATE.with(|state| {
        let state = state.borrow();
        let map = decimal_separator_map();
        match &state.locale {
            Some(locale) => map.get(locale.as_str()).unwrap_or(&".").to_string(),
            None => ".".to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_separators() {
        deactivate();
        assert_eq!(thousands_separator(), ",");
        assert_eq!(decimal_separator(), ".");
    }

    #[test]
    fn test_null_translation() {
        deactivate();
        assert_eq!(gettext("hello"), "hello");
        assert_eq!(pgettext("ctx", "hello"), "hello");
        assert_eq!(ngettext("1 item", "%d items", 1), "1 item");
        assert_eq!(ngettext("1 item", "%d items", 2), "%d items");
    }
}
