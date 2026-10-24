use crate::{CargoRustcEnvMap, CargoWarning, VergenKey, constants::VERGEN_IDEMPOTENT_DEFAULT};
use std::env;

/// Add a [`VergenKey`] entry as a default string into the [`CargoRustcEnvMap`].
/// The value is either from an environment variable override or [`crate::constants::VERGEN_IDEMPOTENT_DEFAULT`]
///
/// # Example
/// ```
/// # use std::collections::BTreeMap;
/// # use temp_env::with_var;
/// # use smolgen::{add_default_map_entry, CargoRustcEnvMap, CargoWarning, VergenKey};
/// with_var("VERGEN_BUILD_DATE", Some("my own date"), || {
///     let mut map: CargoRustcEnvMap = BTreeMap::new();
///     let mut warning: CargoWarning = vec![];
#[cfg_attr(
    feature = "build",
    doc = r"    add_default_map_entry(false, VergenKey::BuildDate, &mut map, &mut warning);
assert_eq!(1, map.len());
assert_eq!(1, warning.len());"
)]
/// });
/// ```
///
pub fn add_default_map_entry(
    idempotent: bool,
    key: VergenKey,
    map: &mut CargoRustcEnvMap,
    warnings: &mut CargoWarning,
) {
    if let Ok(value) = env::var(key.name()) {
        add_map_entry(key, value, map);
        warnings.push(format!("{} overidden", key.name()));
    } else if idempotent {
        add_map_entry(key, VERGEN_IDEMPOTENT_DEFAULT, map);
        warnings.push(format!("{} set to default", key.name()));
    } else {
        warnings.push(format!("Unable to set {}", key.name()));
    }
}

/// Add a [`VergenKey`] entry as a string into the [`CargoRustcEnvMap`].
///
/// # Example
/// ```
/// # use std::collections::BTreeMap;
/// # use smolgen::{add_map_entry, CargoRustcEnvMap, VergenKey};
/// let mut map: CargoRustcEnvMap = BTreeMap::new();
#[cfg_attr(
    feature = "build",
    doc = r#"add_map_entry(VergenKey::BuildDate, "test", &mut map);
assert_eq!(1, map.len());"#
)]
/// ```
///
pub fn add_map_entry<T>(key: VergenKey, value: T, map: &mut CargoRustcEnvMap)
where
    T: Into<String>,
{
    let _old = map.insert(key, value.into());
}

/// Count the number of idempotent entries in a [`CargoRustcEnvMap`]
///
/// **NOTE** - This is mainly used for testing.
///
/// # Example
///
/// ```
/// # use std::collections::BTreeMap;
/// # use smolgen::{count_idempotent, CargoRustcEnvMap, VergenKey, constants::VERGEN_IDEMPOTENT_DEFAULT};
/// #
/// let mut map: CargoRustcEnvMap = BTreeMap::new();
/// assert_eq!(0, count_idempotent(&map));
#[cfg_attr(
    feature = "build",
    doc = r"_ = map.insert(VergenKey::BuildDate, VERGEN_IDEMPOTENT_DEFAULT.to_string());"
)]
#[cfg_attr(feature = "build", doc = r"assert_eq!(1, count_idempotent(&map));")]
/// ```
///
#[must_use]
pub fn count_idempotent(map: &CargoRustcEnvMap) -> usize {
    map.values()
        .filter(|x| *x == VERGEN_IDEMPOTENT_DEFAULT)
        .count()
}

/// Read the git SHA and dirty flag from a `.cargo_vcs_info.json` file at
/// `CARGO_MANIFEST_DIR`, if one is present.
///
/// This is the file [cargo includes in published crate tarballs][vcs-info], which
/// lets the git SHA and dirty flag be recovered when building from a package where
/// no `.git` directory exists (e.g. `cargo install` or a crates.io source).
///
/// Returns `Some((sha, dirty))` on success, or `None` if the file is absent or
/// cannot be parsed. `dirty` defaults to `false` when the field is missing.
///
/// [vcs-info]: https://doc.rust-lang.org/cargo/commands/cargo-package.html#cargo_vcs_infojson-format
#[cfg(feature = "vcs_info")]
#[must_use]
pub fn vcs_info() -> Option<(String, bool)> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")?;
    let path = std::path::Path::new(&manifest_dir).join(".cargo_vcs_info.json");
    let contents = std::fs::read_to_string(path).ok()?;
    let json = json::parse(&contents)?;
    let git = json.get("git")?;
    let sha = git.get("sha1")?.as_str()?.to_string();
    let dirty = git
        .get("dirty")
        .and_then(json::Value::as_bool)
        .unwrap_or(false);
    Some((sha, dirty))
}

/// A minimal JSON reader, enough for `.cargo_vcs_info.json`.
#[cfg(feature = "vcs_info")]
mod json {
    use std::{iter::Peekable, str::Chars};

    pub(crate) enum Value {
        Null,
        Bool(bool),
        Number,
        String(String),
        Array,
        Object(Vec<(String, Value)>),
    }

    impl Value {
        pub(crate) fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Object(entries) => {
                    entries.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v)
                }
                _ => None,
            }
        }

        pub(crate) fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(s) => Some(s),
                _ => None,
            }
        }

        pub(crate) fn as_bool(&self) -> Option<bool> {
            match self {
                Value::Bool(b) => Some(*b),
                _ => None,
            }
        }
    }

    /// Parses a complete JSON document.
    pub(crate) fn parse(input: &str) -> Option<Value> {
        let mut chars = input.chars().peekable();
        let value = parse_value(&mut chars)?;
        skip_ws(&mut chars);
        chars.next().is_none().then_some(value)
    }

    fn skip_ws(chars: &mut Peekable<Chars<'_>>) {
        while chars
            .next_if(|c| matches!(c, ' ' | '\t' | '\n' | '\r'))
            .is_some()
        {}
    }

    fn eat(chars: &mut Peekable<Chars<'_>>, expected: char) -> Option<()> {
        skip_ws(chars);
        chars.next_if_eq(&expected).map(drop)
    }

    fn parse_value(chars: &mut Peekable<Chars<'_>>) -> Option<Value> {
        skip_ws(chars);
        match *chars.peek()? {
            '{' => {
                let _ = chars.next();
                let mut entries = Vec::new();
                if eat(chars, '}').is_some() {
                    return Some(Value::Object(entries));
                }
                loop {
                    skip_ws(chars);
                    let key = parse_string(chars)?;
                    eat(chars, ':')?;
                    entries.push((key, parse_value(chars)?));
                    if eat(chars, ',').is_none() {
                        eat(chars, '}')?;
                        return Some(Value::Object(entries));
                    }
                }
            }
            '[' => {
                let _ = chars.next();
                if eat(chars, ']').is_some() {
                    return Some(Value::Array);
                }
                loop {
                    let _ = parse_value(chars)?;
                    if eat(chars, ',').is_none() {
                        eat(chars, ']')?;
                        return Some(Value::Array);
                    }
                }
            }
            '"' => parse_string(chars).map(Value::String),
            't' => literal(chars, "true", Value::Bool(true)),
            'f' => literal(chars, "false", Value::Bool(false)),
            'n' => literal(chars, "null", Value::Null),
            '-' | '0'..='9' => {
                let mut number = String::new();
                while let Some(c) =
                    chars.next_if(|c| matches!(c, '0'..='9' | '-' | '+' | '.' | 'e' | 'E'))
                {
                    number.push(c);
                }
                valid_number(&number).then_some(Value::Number)
            }
            _ => None,
        }
    }

    /// Checks the JSON number grammar: `-?(0|[1-9][0-9]*)(.[0-9]+)?([eE][+-]?[0-9]+)?`.
    fn valid_number(number: &str) -> bool {
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        let number = number.strip_prefix('-').unwrap_or(number);
        let (mantissa, exponent) = match number.find(['e', 'E']) {
            Some(i) => (&number[..i], Some(&number[i + 1..])),
            None => (number, None),
        };
        let (int, frac) = match mantissa.split_once('.') {
            Some((int, frac)) => (int, Some(frac)),
            None => (mantissa, None),
        };
        digits(int)
            && (int == "0" || !int.starts_with('0'))
            && frac.is_none_or(digits)
            && exponent.is_none_or(|e| digits(e.strip_prefix(['+', '-']).unwrap_or(e)))
    }

    fn literal(chars: &mut Peekable<Chars<'_>>, word: &str, value: Value) -> Option<Value> {
        word.chars()
            .all(|c| chars.next() == Some(c))
            .then_some(value)
    }

    fn parse_string(chars: &mut Peekable<Chars<'_>>) -> Option<String> {
        if chars.next()? != '"' {
            return None;
        }
        let mut out = String::new();
        loop {
            match chars.next()? {
                '"' => return Some(out),
                '\\' => match chars.next()? {
                    c @ ('"' | '\\' | '/') => out.push(c),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let hex = |chars: &mut Peekable<Chars<'_>>| -> Option<u32> {
                            let digits: String =
                                (0..4).map(|_| chars.next()).collect::<Option<_>>()?;
                            u32::from_str_radix(&digits, 16).ok()
                        };
                        let high = hex(chars)?;
                        let code = if (0xD800..0xDC00).contains(&high) {
                            if chars.next()? != '\\' || chars.next()? != 'u' {
                                return None;
                            }
                            let low = hex(chars)?;
                            0x10000 + ((high - 0xD800) << 10) + low.checked_sub(0xDC00)?
                        } else {
                            high
                        };
                        out.push(char::from_u32(code)?);
                    }
                    _ => return None,
                },
                c if c < ' ' => return None,
                c => out.push(c),
            }
        }
    }
}
