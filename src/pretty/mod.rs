// Copyright (c) 2022 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! A pretty printer for `VERGEN_*` environment variables (formerly `vergen-pretty`)
//!
//! Because `cargo` doesn't pass compile time environment variables to dependencies,
//! the [`vergen_pretty_env`] macro embeds a map of all the possible `vergen` environment variables with
//! [`option_env!`](std::option_env!).  Values not set in by your `build.rs` are skipped
//! when pretty-printing the output.
//!
//! # Example
//! ```
//! # use smolgen::Result;
//! # use std::{collections::BTreeMap, io::Write};
//! # use smolgen::pretty::{vergen_pretty_env, Pretty};
//! # fn has_value(
//! #     tuple: (&&'static str, &Option<&'static str>),
//! # ) -> Option<(&'static str, &'static str)> {
//! #     let (key, value) = tuple;
//! #     if value.is_some() {
//! #         Some((*key, value.unwrap_or_default()))
//! #     } else {
//! #         None
//! #     }
//! # }
//! # fn is_empty(map: &BTreeMap<&'static str, Option<&'static str>>) -> bool {
//! #     map.iter().filter_map(has_value).count() == 0
//! # }
//! # fn main() -> Result<()> {
//! let mut stdout = vec![];
//! # let map = vergen_pretty_env!();
//! # let empty = is_empty(&map);
//! Pretty::builder()
//!     .env(vergen_pretty_env!())
//!     .build()
//!     .display(&mut stdout)?;
//! # if empty {
//! #    assert_eq!(stdout.len(), 0);
//! # } else {
//! assert_ne!(stdout.len(), 0);
//! # }
//! #     Ok(())
//! # }
//! ```
//!
//! See the [`Pretty`] documentation for more examples
//!
#![cfg_attr(
    feature = "header",
    doc = r"If you enable the header feature, you can also use the [`header()`] function
with the associated [`Config`] as a convenience wrapper around [`Pretty`].

# Example
```
# use smolgen::Result;
# use smolgen::pretty::{Config, header, vergen_pretty_env};"
)]
#![cfg_attr(feature = "color", doc = r"# use smolgen::pretty::Style;")]
#![cfg_attr(
    feature = "header",
    doc = r"
#
# pub fn main() -> Result<()> {
let mut buf = vec![];
let config = Config::builder()"
)]
#![cfg_attr(
    all(feature = "color", feature = "header"),
    doc = r"    .style(Style::new().green())"
)]
#![cfg_attr(
    feature = "header",
    doc = r#"
    .prefix("HEADER_PREFIX")
    .env(vergen_pretty_env!())
    .suffix("HEADER_SUFFIX")
    .build();
assert!(header(&config, Some(&mut buf)).is_ok());
assert_ne!(buf.len(), 0);
#     Ok(())
# }
```
"#
)]
//!
//! ## Features
//! `vergen-pretty` has two feature toggles allowing you to customize your output. No features are enabled by default.
//! You **must** specifically enable the features you wish to use.
//!
//! | Feature | Enables |
//! | ------- | ------- |
//! |  color  | Colorize output, allow configuration of coloring via [`console`] |
//! |  header | Generate pretty printed header output based on the given [`Config`] |
//! |  trace  | Enable support for [`tracing`](https://docs.rs/tracing/latest/tracing/) output |
//!

#[cfg(feature = "header")]
mod header;
mod printer;
mod utils;

#[cfg(feature = "header")]
pub use self::header::Config;
#[cfg(feature = "header")]
pub use self::header::Env;
#[cfg(feature = "header")]
pub use self::header::header;
pub use self::printer::Pretty;
#[cfg(any(feature = "serde", feature = "rkyv"))]
pub use self::printer::feature::PrettyExt;
#[cfg(all(feature = "rkyv", feature = "color"))]
pub use self::printer::feature::rkyv_support::StyleWith;
pub use self::printer::prefix::Prefix;
pub use self::printer::suffix::Suffix;
#[cfg(feature = "color")]
#[doc(inline)]
pub use console::Style;
#[cfg(feature = "trace")]
#[doc(inline)]
pub use tracing::Level;

#[doc(inline)]
pub use crate::vergen_pretty_env;

/// Used to initialize `env` in [`Pretty`](self::Pretty)
///
/// Because `cargo` doesn't pass compile time environment variables to dependencies,
/// this macro embeds a map of all the possible `vergen` environment variables with
/// [`option_env!`](std::option_env!).  Non-set values are skipped when pretty-printing the output.
///
/// # Example
/// ```
/// # use smolgen::Result;
/// # use std::{collections::BTreeMap, io::Write};
/// # use smolgen::pretty::{vergen_pretty_env, Pretty};
/// #
/// # fn main() -> Result<()> {
/// let mut stdout = vec![];
/// Pretty::builder()
///     .env(vergen_pretty_env!())
///     .build()
///     .display(&mut stdout)?;
/// #     Ok(())
/// # }
#[macro_export]
macro_rules! vergen_pretty_env {
    () => {{
        use std::collections::BTreeMap;
        let mut map = BTreeMap::new();
        let _old = map.insert("VERGEN_BUILD_DATE", option_env!("VERGEN_BUILD_DATE"));
        let _old = map.insert(
            "VERGEN_BUILD_TIMESTAMP",
            option_env!("VERGEN_BUILD_TIMESTAMP"),
        );
        let _old = map.insert("VERGEN_CARGO_DEBUG", option_env!("VERGEN_CARGO_DEBUG"));
        let _old = map.insert(
            "VERGEN_CARGO_FEATURES",
            option_env!("VERGEN_CARGO_FEATURES"),
        );
        let _old = map.insert(
            "VERGEN_CARGO_OPT_LEVEL",
            option_env!("VERGEN_CARGO_OPT_LEVEL"),
        );
        let _old = map.insert(
            "VERGEN_CARGO_TARGET_TRIPLE",
            option_env!("VERGEN_CARGO_TARGET_TRIPLE"),
        );
        let _old = map.insert(
            "VERGEN_CARGO_DEPENDENCIES",
            option_env!("VERGEN_CARGO_DEPENDENCIES"),
        );
        let _old = map.insert("VERGEN_GIT_BRANCH", option_env!("VERGEN_GIT_BRANCH"));
        let _old = map.insert(
            "VERGEN_GIT_COMMIT_AUTHOR_EMAIL",
            option_env!("VERGEN_GIT_COMMIT_AUTHOR_EMAIL"),
        );
        let _old = map.insert(
            "VERGEN_GIT_COMMIT_AUTHOR_NAME",
            option_env!("VERGEN_GIT_COMMIT_AUTHOR_NAME"),
        );
        let _old = map.insert(
            "VERGEN_GIT_COMMIT_COUNT",
            option_env!("VERGEN_GIT_COMMIT_COUNT"),
        );
        let _old = map.insert(
            "VERGEN_GIT_COMMIT_DATE",
            option_env!("VERGEN_GIT_COMMIT_DATE"),
        );
        let _old = map.insert(
            "VERGEN_GIT_COMMIT_MESSAGE",
            option_env!("VERGEN_GIT_COMMIT_MESSAGE"),
        );
        let _old = map.insert(
            "VERGEN_GIT_COMMIT_TIMESTAMP",
            option_env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
        );
        let _old = map.insert("VERGEN_GIT_DESCRIBE", option_env!("VERGEN_GIT_DESCRIBE"));
        let _old = map.insert("VERGEN_GIT_SHA", option_env!("VERGEN_GIT_SHA"));
        let _old = map.insert("VERGEN_GIT_DIRTY", option_env!("VERGEN_GIT_DIRTY"));
        let _old = map.insert("VERGEN_RUSTC_CHANNEL", option_env!("VERGEN_RUSTC_CHANNEL"));
        let _old = map.insert(
            "VERGEN_RUSTC_COMMIT_DATE",
            option_env!("VERGEN_RUSTC_COMMIT_DATE"),
        );
        let _old = map.insert(
            "VERGEN_RUSTC_COMMIT_HASH",
            option_env!("VERGEN_RUSTC_COMMIT_HASH"),
        );
        let _old = map.insert(
            "VERGEN_RUSTC_HOST_TRIPLE",
            option_env!("VERGEN_RUSTC_HOST_TRIPLE"),
        );
        let _old = map.insert(
            "VERGEN_RUSTC_LLVM_VERSION",
            option_env!("VERGEN_RUSTC_LLVM_VERSION"),
        );
        let _old = map.insert("VERGEN_RUSTC_SEMVER", option_env!("VERGEN_RUSTC_SEMVER"));
        let _old = map.insert("VERGEN_SYSINFO_NAME", option_env!("VERGEN_SYSINFO_NAME"));
        let _old = map.insert(
            "VERGEN_SYSINFO_OS_VERSION",
            option_env!("VERGEN_SYSINFO_OS_VERSION"),
        );
        let _old = map.insert("VERGEN_SYSINFO_USER", option_env!("VERGEN_SYSINFO_USER"));
        let _old = map.insert(
            "VERGEN_SYSINFO_TOTAL_MEMORY",
            option_env!("VERGEN_SYSINFO_TOTAL_MEMORY"),
        );
        let _old = map.insert(
            "VERGEN_SYSINFO_CPU_VENDOR",
            option_env!("VERGEN_SYSINFO_CPU_VENDOR"),
        );
        let _old = map.insert(
            "VERGEN_SYSINFO_CPU_CORE_COUNT",
            option_env!("VERGEN_SYSINFO_CPU_CORE_COUNT"),
        );
        let _old = map.insert(
            "VERGEN_SYSINFO_CPU_NAME",
            option_env!("VERGEN_SYSINFO_CPU_NAME"),
        );
        let _old = map.insert(
            "VERGEN_SYSINFO_CPU_BRAND",
            option_env!("VERGEN_SYSINFO_CPU_BRAND"),
        );
        let _old = map.insert(
            "VERGEN_SYSINFO_CPU_FREQUENCY",
            option_env!("VERGEN_SYSINFO_CPU_FREQUENCY"),
        );
        map
    }};
    ( $( $x:expr ),* ) => {{
        {
            let mut map = $crate::vergen_pretty_env!();
            $(
                let _old = map.insert($x, option_env!($x));
            )*
            map
        }
    }};
}
