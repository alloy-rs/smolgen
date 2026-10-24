// Copyright (c) 2022 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

#[allow(clippy::ref_option_ref)]
pub(crate) fn has_value(
    tuple: (&&'static str, &Option<&'static str>),
) -> Option<(&'static str, &'static str)> {
    let (key, value) = tuple;
    if value.is_some() {
        Some((*key, value.unwrap_or_default()))
    } else {
        None
    }
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn split_key(tuple: (&str, &str)) -> Option<(Vec<String>, String)> {
    let (key, value) = tuple;
    let key = key.to_ascii_lowercase();
    if key.starts_with("vergen") {
        let kv_vec: Vec<String> = key.split('_').filter_map(not_vergen).collect();
        Some((kv_vec, value.to_string()))
    } else {
        Some((vec![key], value.to_string()))
    }
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn split_kv(tuple: (Vec<String>, String)) -> Option<(String, String, String)> {
    let (mut kv, v) = tuple;
    if kv.len() >= 2 {
        let category = kv.remove(0);
        let label = kv
            .into_iter()
            .map(caps_proper)
            .fold(String::new(), |a, b| a + " " + &b);
        Some((category, label, v))
    } else {
        Some(("custom".to_string(), kv[0].clone(), v))
    }
}

pub(crate) fn not_vergen(part: &str) -> Option<String> {
    if part == "vergen" {
        None
    } else {
        Some(part.to_string())
    }
}

#[allow(clippy::needless_pass_by_value)]
fn caps_proper(val: String) -> String {
    if val == "cpu" || val == "os" || val == "llvm" || val == "sha" {
        val.to_ascii_uppercase()
    } else {
        title_case(&val)
    }
}

#[cfg(feature = "serde")]
pub(crate) fn snake_case(val: &str) -> String {
    words(val)
        .iter()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

fn title_case(val: &str) -> String {
    words(val)
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |c| {
                c.to_uppercase()
                    .chain(chars.as_str().to_lowercase().chars())
                    .collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Splits `val` into words, keeping empty ones: on `_`, `-` and ` `, between lowercase and
/// uppercase, between letters and digits, and before the last uppercase letter of an acronym
/// followed by a lowercase letter.
fn words(val: &str) -> Vec<&str> {
    if val.is_empty() {
        return Vec::new();
    }
    let chars: Vec<_> = val.char_indices().collect();
    let mut words = Vec::new();
    let mut start = 0;
    for (i, &(idx, c)) in chars.iter().enumerate() {
        if matches!(c, '_' | '-' | ' ') {
            words.push(&val[start..idx]);
            start = idx + 1;
        } else if let Some(&(next_idx, next)) = chars.get(i + 1) {
            let letter = |c: char| c.is_lowercase() || c.is_uppercase();
            let split = (c.is_lowercase() && next.is_uppercase())
                || (letter(c) && next.is_ascii_digit())
                || (c.is_ascii_digit() && letter(next))
                || (c.is_uppercase()
                    && next.is_uppercase()
                    && chars.get(i + 2).is_some_and(|&(_, c)| c.is_lowercase()));
            if split {
                words.push(&val[start..next_idx]);
                start = next_idx;
            }
        }
    }
    words.push(&val[start..]);
    words
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::{has_value, split_key, split_kv};
    use std::collections::BTreeMap;

    pub(crate) const TEST_PREFIX_SUFFIX: &str = r"██████╗ ██████╗ ███████╗████████╗████████╗██╗   ██╗
██╔══██╗██╔══██╗██╔════╝╚══██╔══╝╚══██╔══╝╚██╗ ██╔╝
██████╔╝██████╔╝█████╗     ██║      ██║    ╚████╔╝ 
██╔═══╝ ██╔══██╗██╔══╝     ██║      ██║     ╚██╔╝  
██║     ██║  ██║███████╗   ██║      ██║      ██║   
╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝      ╚═╝      ╚═╝       

4a61736f6e204f7a696173
";

    pub(crate) fn is_empty(map: &BTreeMap<&'static str, Option<&'static str>>) -> bool {
        map.iter().filter_map(has_value).count() == 0
    }

    #[test]
    fn has_value_none_is_none() {
        assert!(has_value((&"test", &None)).is_none());
    }

    #[test]
    fn split_key_no_vergen_is_not_split() {
        let (split_key, value) = split_key(("test_k", "test_v")).unwrap_or_default();
        assert_eq!(1, split_key.len());
        assert_eq!("test_k", split_key[0]);
        assert_eq!("test_v", value);
    }

    #[test]
    fn split_kv_too_short_is_custom() {
        let (category, label, value) =
            split_kv((vec!["test_k".to_string()], "test_v".to_string())).unwrap_or_default();
        assert_eq!("custom", category);
        assert_eq!("test_k", label);
        assert_eq!("test_v", value);
    }
}
