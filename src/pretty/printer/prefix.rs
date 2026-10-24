// Copyright (c) 2022 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use crate::Result;
#[cfg(feature = "color")]
use console::Style;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io::Write;
#[cfg(feature = "trace")]
use tracing::Level;

/// Configure prefix output for [`Pretty`](crate::pretty::Pretty)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "trace"), derive(Default))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Prefix {
    pub(crate) lines: Vec<String>,
    #[cfg(feature = "color")]
    #[cfg_attr(feature = "serde", serde(skip))]
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::Map<crate::pretty::printer::feature::rkyv_support::StyleWith>))]
    pub(crate) style: Option<Style>,
    #[cfg(feature = "trace")]
    #[cfg_attr(feature = "serde", serde(skip, default = "default_level"))]
    #[cfg_attr(
        feature = "rkyv",
        rkyv(with = crate::pretty::printer::feature::rkyv_support::LevelWith)
    )]
    pub(crate) level: Level,
}

#[cfg(feature = "trace")]
impl Default for Prefix {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            #[cfg(feature = "color")]
            style: None,
            level: Level::INFO,
        }
    }
}

builder! {
    PrefixBuilder => Prefix {
        /// The prefix lines to output
        lines: Vec<String>,
        /// The tracing [`Level`] to output the prefix at
        #[cfg(feature = "trace")]
        level: Level,
    }
}

#[cfg(feature = "color")]
impl PrefixBuilder {
    /// The [`Style`] to apply to the output lines
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.0.style = Some(style);
        self
    }

    /// The [`Style`] to apply to the output lines
    #[must_use]
    pub fn maybe_style(mut self, style: Option<Style>) -> Self {
        self.0.style = style;
        self
    }
}

impl Prefix {
    /// Output the `vergen` environment variables that are set in table format
    ///
    /// # Errors
    /// * The [`writeln!`](std::writeln!) macro can throw a [`std::io::Error`]
    ///
    pub(crate) fn display<T>(&self, writer: &mut T) -> Result<()>
    where
        T: Write + ?Sized,
    {
        self.inner_display(writer)?;
        writeln!(writer)?;
        Ok(())
    }

    #[cfg(not(feature = "color"))]
    fn inner_display<T>(&self, writer: &mut T) -> Result<()>
    where
        T: Write + ?Sized,
    {
        for line in &self.lines {
            writeln!(writer, "{line}")?;
        }
        Ok(())
    }
}

#[cfg(all(feature = "serde", feature = "trace"))]
fn default_level() -> Level {
    Level::INFO
}

#[cfg(test)]
mod test {
    use crate::Result;
    use crate::{
        pretty::{Prefix, Pretty, utils::test_utils::TEST_PREFIX_SUFFIX},
        vergen_pretty_env,
    };
    use std::io::Write;

    #[test]
    #[allow(clippy::clone_on_copy, clippy::redundant_clone)]
    fn prefix_clone_works() {
        let prefix = Prefix::builder()
            .lines(TEST_PREFIX_SUFFIX.lines().map(str::to_string).collect())
            .build();
        let another = prefix.clone();
        assert_eq!(prefix, another);
    }

    #[test]
    fn prefix_debug_works() -> Result<()> {
        let prefix = Prefix::builder()
            .lines(TEST_PREFIX_SUFFIX.lines().map(str::to_string).collect())
            .build();
        let mut buf = vec![];
        write!(buf, "{prefix:?}")?;
        assert_ne!(buf.len(), 0);
        Ok(())
    }

    #[test]
    fn display_prefix_works() -> Result<()> {
        let mut stdout = vec![];
        let map = vergen_pretty_env!();
        let prefix = Prefix::builder()
            .lines(TEST_PREFIX_SUFFIX.lines().map(str::to_string).collect())
            .build();
        let fmt = Pretty::builder().env(map).prefix(prefix).build();
        fmt.display(&mut stdout)?;
        assert_ne!(stdout.len(), 0);
        Ok(())
    }
}
