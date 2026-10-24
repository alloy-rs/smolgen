// Copyright (c) 2022 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use crate::{
    AddEntries, CargoRerunIfChanged, CargoRustcEnvMap, CargoWarning, DefaultConfig, VergenKey,
    add_default_map_entry, add_map_entry,
    constants::{
        RUSTC_CHANNEL_NAME, RUSTC_COMMIT_DATE, RUSTC_COMMIT_HASH, RUSTC_HOST_TRIPLE_NAME,
        RUSTC_LLVM_VERSION, RUSTC_SEMVER_NAME,
    },
};
use crate::{Error, Result};
use std::{collections::BTreeMap, env, ffi::OsString, process::Command};

/// The `VERGEN_RUSTC_*` configuration features
///
/// **NOTE** - All rustc instructions are considered deterministic.  If you change
/// the version of rustc you are compiling with, these values should change if
/// being used in the generated binary.
///
/// | Variable | Sample |
/// | -------  | ------ |
/// | `VERGEN_RUSTC_CHANNEL` | nightly |
/// | `VERGEN_RUSTC_COMMIT_DATE` | 2021-02-24 |
/// | `VERGEN_RUSTC_COMMIT_HASH` | a8486b64b0c87dabd045453b6c81500015d122d6 |
/// | `VERGEN_RUSTC_HOST_TRIPLE` | apple-darwin |
/// | `VERGEN_RUSTC_LLVM_VERSION` | 11.0 |
/// | `VERGEN_RUSTC_SEMVER` | 1.52.0-nightly |
///
/// # Example
/// Emit all of the rustc instructions
///
/// ```
/// # use smolgen::Result;
/// # use smolgen::Emitter;
/// # use smolgen::Rustc;
/// #
/// # fn main() -> Result<()> {
/// let rustc = Rustc::all_rustc();
/// Emitter::default().add_instructions(&rustc)?.emit();
/// #   Ok(())
/// # }
/// ```
///
/// Emit some of the rustc instructions
///
/// ```
/// # use smolgen::Result;
/// # use smolgen::Emitter;
/// # use smolgen::Rustc;
/// #
/// # fn main() -> Result<()> {
/// let rustc = Rustc::builder().channel(true).semver(true).build();
/// Emitter::default().add_instructions(&rustc)?.emit();
/// #   Ok(())
/// # }
/// ```
///
/// Override output with your own value
///
/// ```
/// # use smolgen::Result;
/// # use std::env;
/// # use smolgen::Emitter;
/// # use smolgen::Rustc;
/// #
/// # fn main() -> Result<()> {
/// temp_env::with_var("VERGEN_RUSTC_CHANNEL", Some("this is the channel I want output"), || {
///     let result = || -> Result<()> {
///         let rustc = Rustc::builder().channel(true).semver(true).build();
///         Emitter::default().add_instructions(&rustc)?.emit();   
///         Ok(())  
///     }();
///     assert!(result.is_ok());
/// });
/// #   Ok(())
/// # }
/// ```
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Rustc {
    /// Whether the `all` constructor created this value.
    all: bool,
    #[cfg(test)]
    str_to_test: Option<&'static str>,
    channel: bool,
    commit_date: bool,
    commit_hash: bool,
    host_triple: bool,
    llvm_version: bool,
    semver: bool,
}

builder! {
    RustcBuilder => Rustc {
        /// Enable the rustc channel
        channel: bool,
        /// Enable the rustc commit date
        commit_date: bool,
        /// Enable the rustc SHA
        commit_hash: bool,
        /// Enable rustc host triple
        host_triple: bool,
        /// Enable rustc LLVM version
        llvm_version: bool,
        /// Enable the rustc semver
        semver: bool,
    }
}

impl RustcBuilder {
    #[cfg(test)]
    #[must_use]
    pub fn str_to_test(mut self, str_to_test: &'static str) -> Self {
        self.0.str_to_test = Some(str_to_test);
        self
    }

    #[cfg(test)]
    #[must_use]
    pub fn maybe_str_to_test(mut self, str_to_test: Option<&'static str>) -> Self {
        self.0.str_to_test = str_to_test;
        self
    }
}

impl Rustc {
    /// Enable all of the `VERGEN_RUSTC_*` options
    #[must_use]
    pub fn all_rustc() -> Self {
        Self::all().build()
    }

    /// Convenience method to setup the builder with all of the `VERGEN_RUSTC_*` instructions on
    pub fn all() -> RustcBuilder {
        RustcBuilder(Self {
            all: true,
            channel: true,
            commit_date: true,
            commit_hash: true,
            host_triple: true,
            llvm_version: true,
            semver: true,
            #[cfg(test)]
            str_to_test: None,
        })
    }

    fn any(self) -> bool {
        self.channel
            || self.commit_date
            || self.commit_hash
            || self.host_triple
            || self.llvm_version
            || self.semver
    }

    #[cfg(not(test))]
    fn add_rustc_map_entries(
        self,
        cargo_rustc_env: &mut CargoRustcEnvMap,
        cargo_warning: &mut CargoWarning,
    ) -> Result<()> {
        self.add_rustc_to_map(version_meta(), cargo_rustc_env, cargo_warning)
    }

    #[cfg(test)]
    fn add_rustc_map_entries(
        self,
        cargo_rustc_env: &mut CargoRustcEnvMap,
        cargo_warning: &mut CargoWarning,
    ) -> Result<()> {
        let vm = if let Some(rustc_str) = self.str_to_test {
            version_meta_for(rustc_str)
        } else {
            version_meta()
        };
        self.add_rustc_to_map(vm, cargo_rustc_env, cargo_warning)
    }

    #[allow(clippy::too_many_lines)]
    fn add_rustc_to_map(
        self,
        rustc_res: Result<VersionMeta>,
        cargo_rustc_env: &mut CargoRustcEnvMap,
        cargo_warning: &mut CargoWarning,
    ) -> Result<()> {
        let rustc = rustc_res?;

        if self.channel {
            if let Ok(_value) = env::var(RUSTC_CHANNEL_NAME) {
                add_default_map_entry(
                    false,
                    VergenKey::RustcChannel,
                    cargo_rustc_env,
                    cargo_warning,
                );
            } else {
                add_map_entry(VergenKey::RustcChannel, rustc.channel, cargo_rustc_env);
            }
        }

        if self.commit_date {
            if let Ok(_value) = env::var(RUSTC_COMMIT_DATE) {
                add_default_map_entry(
                    false,
                    VergenKey::RustcCommitDate,
                    cargo_rustc_env,
                    cargo_warning,
                );
            } else if let Some(commit_date) = rustc.commit_date {
                add_map_entry(VergenKey::RustcCommitDate, commit_date, cargo_rustc_env);
            } else {
                add_default_map_entry(
                    false,
                    VergenKey::RustcCommitDate,
                    cargo_rustc_env,
                    cargo_warning,
                );
            }
        }

        if self.commit_hash {
            if let Ok(_value) = env::var(RUSTC_COMMIT_HASH) {
                add_default_map_entry(
                    false,
                    VergenKey::RustcCommitHash,
                    cargo_rustc_env,
                    cargo_warning,
                );
            } else if let Some(commit_hash) = rustc.commit_hash {
                add_map_entry(VergenKey::RustcCommitHash, commit_hash, cargo_rustc_env);
            } else {
                add_default_map_entry(
                    false,
                    VergenKey::RustcCommitHash,
                    cargo_rustc_env,
                    cargo_warning,
                );
            }
        }

        if self.host_triple {
            if let Ok(_value) = env::var(RUSTC_HOST_TRIPLE_NAME) {
                add_default_map_entry(
                    false,
                    VergenKey::RustcHostTriple,
                    cargo_rustc_env,
                    cargo_warning,
                );
            } else {
                add_map_entry(VergenKey::RustcHostTriple, rustc.host, cargo_rustc_env);
            }
        }

        if self.llvm_version {
            if let Ok(_value) = env::var(RUSTC_LLVM_VERSION) {
                add_default_map_entry(
                    false,
                    VergenKey::RustcLlvmVersion,
                    cargo_rustc_env,
                    cargo_warning,
                );
            } else if let Some(llvm_version) = rustc.llvm_version {
                add_map_entry(VergenKey::RustcLlvmVersion, llvm_version, cargo_rustc_env);
            } else {
                add_default_map_entry(
                    false,
                    VergenKey::RustcLlvmVersion,
                    cargo_rustc_env,
                    cargo_warning,
                );
            }
        }

        if self.semver {
            if let Ok(_value) = env::var(RUSTC_SEMVER_NAME) {
                add_default_map_entry(
                    false,
                    VergenKey::RustcSemver,
                    cargo_rustc_env,
                    cargo_warning,
                );
            } else {
                add_map_entry(VergenKey::RustcSemver, rustc.semver, cargo_rustc_env);
            }
        }

        Ok(())
    }

    #[cfg(test)]
    fn with_rustc_str(&mut self, rustc_str: &'static str) -> &mut Self {
        self.str_to_test = Some(rustc_str);
        self
    }
}

/// The parts of `rustc -vV` output that [`Rustc`] emits.
struct VersionMeta {
    semver: String,
    channel: &'static str,
    commit_hash: Option<String>,
    commit_date: Option<String>,
    host: String,
    llvm_version: Option<String>,
}

/// Runs `$RUSTC -vV` (through `$RUSTC_WRAPPER` when set) and parses its output.
fn version_meta() -> Result<VersionMeta> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let mut cmd = if let Some(wrapper) = env::var_os("RUSTC_WRAPPER").filter(|w| !w.is_empty()) {
        let mut cmd = Command::new(wrapper);
        let _ = cmd.arg(rustc);
        cmd
    } else {
        Command::new(rustc)
    };
    let out = cmd
        .arg("-vV")
        .output()
        .map_err(|e| Error::msg(format!("could not execute command: {e}")))?;
    if !out.status.success() {
        return Err(Error::msg(format!(
            "error from command -- stderr:\n\n{}\n\nstdout:\n\n{}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout),
        )));
    }
    let stdout = String::from_utf8(out.stdout)
        .map_err(|_| Error::msg("invalid UTF-8 output from `rustc -vV`"))?;
    version_meta_for(&stdout)
}

/// Parses `rustc -vV` output.
fn version_meta_for(verbose_version_string: &str) -> Result<VersionMeta> {
    let mut map = BTreeMap::new();
    for line in verbose_version_string.lines().skip(1) {
        if let Some((key, value)) = line.split_once(": ") {
            let _ = map.insert(key, value);
        }
    }
    let expect_key = |key| {
        map.get(key)
            .copied()
            .ok_or_else(|| Error::msg("unexpected `rustc -vV` format"))
    };
    let expect_key_or_unknown =
        |key| expect_key(key).map(|v| (v != "unknown").then(|| v.to_string()));

    let host = expect_key("host")?.to_string();
    let semver = expect_key("release")?;
    let pre = parse_semver(semver)?;
    let channel = match pre.split('.').next().unwrap_or_default() {
        "" => "stable",
        "dev" => "dev",
        "beta" => "beta",
        "nightly" => "nightly",
        x => return Err(Error::msg(format!("unknown pre-release tag: {x}"))),
    };
    let commit_hash = expect_key_or_unknown("commit-hash")?;
    let commit_date = expect_key_or_unknown("commit-date")?;
    let llvm_version = map
        .get("LLVM version")
        .map(|v| parse_llvm_version(v))
        .transpose()?;

    Ok(VersionMeta {
        semver: semver.to_string(),
        channel,
        commit_hash,
        commit_date,
        host,
        llvm_version,
    })
}

/// Validates a SemVer version and returns its pre-release part.
fn parse_semver(version: &str) -> Result<&str> {
    let invalid = || format!("error parsing version: invalid semver `{version}`");
    let (version_pre, build) = version
        .split_once('+')
        .map_or((version, None), |(v, b)| (v, Some(b)));
    let (core, pre) = version_pre.split_once('-').unwrap_or((version_pre, ""));
    let numeric = |part: &str| {
        !part.is_empty()
            && part.bytes().all(|b| b.is_ascii_digit())
            && (part == "0" || !part.starts_with('0'))
    };
    let identifiers = |part: &str, check_numeric: bool| {
        part.split('.').all(|id| {
            !id.is_empty()
                && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && (!check_numeric || !id.bytes().all(|b| b.is_ascii_digit()) || numeric(id))
        })
    };
    let mut parts = core.split('.');
    let valid = (0..3).all(|_| parts.next().is_some_and(numeric))
        && parts.next().is_none()
        && (pre.is_empty() && !version_pre.ends_with('-') || identifiers(pre, true))
        && build.is_none_or(|b| identifiers(b, false));
    if valid {
        Ok(pre)
    } else {
        Err(Error::msg(invalid()))
    }
}

/// Parses LLVM's version the way `rustc_version` does and formats it as `major.minor`.
fn parse_llvm_version(version: &str) -> Result<String> {
    let mut parts = version.split('.').map(|part| -> Result<u64> {
        if part == "0" {
            Ok(0)
        } else if part.starts_with('0') {
            Err(Error::msg(
                "error parsing LLVM's version: a version component must not have leading zeros",
            ))
        } else if part.starts_with('-') || part.starts_with('+') {
            Err(Error::msg(
                "error parsing LLVM's version: a version component must not have a sign",
            ))
        } else {
            part.parse().map_err(|e| {
                Error::msg(format!(
                    "error parsing LLVM's version: error parsing LLVM version component: {e}"
                ))
            })
        }
    });
    let major = parts.next().unwrap_or(Ok(0))?;
    let minor = match parts.next() {
        Some(part) => part?,
        None if major < 4 => {
            return Err(Error::msg(
                "error parsing LLVM's version: LLVM's minor version component is required for versions less than 4.0",
            ));
        }
        None => 0,
    };
    // Like `rustc_version`, validate but ignore a patch component.
    if let Some(Err(e)) = parts.next() {
        return Err(e);
    }
    if parts.next().is_some() {
        return Err(Error::msg(
            "error parsing LLVM's version: too many version components",
        ));
    }
    Ok(format!("{major}.{minor}"))
}

impl AddEntries for Rustc {
    fn add_map_entries(
        &self,
        _idempotent: bool,
        cargo_rustc_env: &mut CargoRustcEnvMap,
        _cargo_rerun_if_changed: &mut CargoRerunIfChanged,
        cargo_warning: &mut CargoWarning,
    ) -> Result<()> {
        if self.any() {
            self.add_rustc_map_entries(cargo_rustc_env, cargo_warning)
        } else {
            Ok(())
        }
    }

    fn add_default_entries(
        &self,
        config: &DefaultConfig,
        cargo_rustc_env_map: &mut CargoRustcEnvMap,
        _cargo_rerun_if_changed: &mut CargoRerunIfChanged,
        cargo_warning: &mut CargoWarning,
    ) -> Result<()> {
        if *config.fail_on_error() {
            let error = Error::msg(format!("{}", config.error()));
            Err(error)
        } else {
            if self.channel {
                add_default_map_entry(
                    false,
                    VergenKey::RustcChannel,
                    cargo_rustc_env_map,
                    cargo_warning,
                );
            }
            if self.commit_date {
                add_default_map_entry(
                    false,
                    VergenKey::RustcCommitDate,
                    cargo_rustc_env_map,
                    cargo_warning,
                );
            }
            if self.commit_hash {
                add_default_map_entry(
                    false,
                    VergenKey::RustcCommitHash,
                    cargo_rustc_env_map,
                    cargo_warning,
                );
            }
            if self.host_triple {
                add_default_map_entry(
                    false,
                    VergenKey::RustcHostTriple,
                    cargo_rustc_env_map,
                    cargo_warning,
                );
            }
            if self.llvm_version {
                add_default_map_entry(
                    false,
                    VergenKey::RustcLlvmVersion,
                    cargo_rustc_env_map,
                    cargo_warning,
                );
            }
            if self.semver {
                add_default_map_entry(
                    false,
                    VergenKey::RustcSemver,
                    cargo_rustc_env_map,
                    cargo_warning,
                );
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::Rustc;
    use crate::Emitter;
    use crate::Result;
    use crate::count_idempotent;
    use serial_test::serial;
    use std::io::Write;
    use temp_env::with_var;

    #[test]
    #[serial]
    #[allow(clippy::clone_on_copy, clippy::redundant_clone)]
    fn rustc_clone_works() {
        let rustc = Rustc::all_rustc();
        let another = rustc.clone();
        assert_eq!(another, rustc);
    }

    #[test]
    #[serial]
    fn rustc_debug_works() -> Result<()> {
        let rustc = Rustc::all_rustc();
        let mut buf = vec![];
        write!(buf, "{rustc:?}")?;
        assert_ne!(buf.len(), 0);
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_default() -> Result<()> {
        let rustc = Rustc::builder().build();
        let emitter = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(0, emitter.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(emitter.cargo_rustc_env_map()));
        assert_eq!(0, emitter.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_all_idempotent() -> Result<()> {
        let rustc = Rustc::all_rustc();
        let config = Emitter::default()
            .idempotent()
            .add_instructions(&rustc)?
            .test_emit();
        assert_eq!(6, config.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(config.cargo_rustc_env_map()));
        assert_eq!(0, config.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_all() -> Result<()> {
        let rustc = Rustc::all_rustc();
        let config = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(6, config.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(config.cargo_rustc_env_map()));
        assert_eq!(0, config.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_commit_date() -> Result<()> {
        let rustc = Rustc::builder().commit_date(true).build();
        let config = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(1, config.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(config.cargo_rustc_env_map()));
        assert_eq!(0, config.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_commit_hash() -> Result<()> {
        let rustc = Rustc::builder().commit_hash(true).build();
        let config = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(1, config.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(config.cargo_rustc_env_map()));
        assert_eq!(0, config.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_host_triple() -> Result<()> {
        let rustc = Rustc::builder().host_triple(true).build();
        let config = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(1, config.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(config.cargo_rustc_env_map()));
        assert_eq!(0, config.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_llvm_version() -> Result<()> {
        let rustc = Rustc::builder().llvm_version(true).build();
        let config = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(1, config.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(config.cargo_rustc_env_map()));
        assert_eq!(0, config.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_semver() -> Result<()> {
        let rustc = Rustc::builder().semver(true).build();
        let config = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(1, config.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(config.cargo_rustc_env_map()));
        assert_eq!(0, config.cargo_warning().len());
        Ok(())
    }

    const NO_LLVM: &str = r"rustc 1.68.0-nightly (270c94e48 2022-12-28)
binary: rustc
commit-hash: 270c94e484e19764a2832ef918c95224eb3f17c7
commit-date: 2022-12-28
host: x86_64-unknown-linux-gnu
release: 1.68.0-nightly
    ";

    #[test]
    #[serial]
    fn no_llvm_in_rustc() -> Result<()> {
        let mut rustc = Rustc::all_rustc();
        let _ = rustc.with_rustc_str(NO_LLVM);
        let emitter = Emitter::default()
            .fail_on_error()
            .add_instructions(&rustc)?
            .test_emit();
        assert_eq!(5, emitter.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(emitter.cargo_rustc_env_map()));
        assert_eq!(1, emitter.cargo_warning().len());
        Ok(())
    }

    const DEV_BUILD: &str = r"rustc 1.68.0-nightly (270c94e48 2022-12-28)
binary: rustc
commit-hash: 270c94e484e19764a2832ef918c95224eb3f17c7
commit-date: 2022-12-28
host: x86_64-unknown-linux-gnu
release: 1.68.0-dev
LLVM version: 15.0.6
    ";

    #[test]
    #[serial]
    fn rustc_dev_build() -> Result<()> {
        let mut rustc = Rustc::all_rustc();
        let _ = rustc.with_rustc_str(DEV_BUILD);
        let emitter = Emitter::default()
            .fail_on_error()
            .add_instructions(&rustc)?
            .test_emit();
        assert_eq!(6, emitter.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(emitter.cargo_rustc_env_map()));
        assert_eq!(0, emitter.cargo_warning().len());
        Ok(())
    }

    const UNKNOWN_BITS: &str = r"rustc 1.68.0-nightly (270c94e48 2022-12-28)
binary: rustc
commit-hash: unknown
commit-date: unknown
host: x86_64-unknown-linux-gnu
release: 1.68.0-dev
LLVM version: 15.0.6
    ";

    #[test]
    #[serial]
    fn rustc_unknown_bits() -> Result<()> {
        let mut rustc = Rustc::all_rustc();
        let _ = rustc.with_rustc_str(UNKNOWN_BITS);
        let emitter = Emitter::default()
            .fail_on_error()
            .add_instructions(&rustc)?
            .test_emit();
        assert_eq!(4, emitter.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(emitter.cargo_rustc_env_map()));
        assert_eq!(2, emitter.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_fails_on_bad_input() -> Result<()> {
        let mut rustc = Rustc::all_rustc();
        let _ = rustc.with_rustc_str("a_bad_rustcvv_string");
        assert!(
            Emitter::default()
                .fail_on_error()
                .add_instructions(&rustc)
                .is_err()
        );
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_warnings_on_bad_input() -> Result<()> {
        let mut rustc = Rustc::all_rustc();
        let _ = rustc.with_rustc_str("a_bad_rustcvv_string");
        let emitter = Emitter::default().add_instructions(&rustc)?.test_emit();
        assert_eq!(0, emitter.cargo_rustc_env_map().len());
        assert_eq!(0, count_idempotent(emitter.cargo_rustc_env_map()));
        assert_eq!(6, emitter.cargo_warning().len());
        Ok(())
    }

    #[test]
    #[serial]
    fn rustc_channel_override_works() {
        with_var("VERGEN_RUSTC_CHANNEL", Some("this is a bad date"), || {
            let result = || -> Result<()> {
                let mut stdout_buf = vec![];
                let rustc = Rustc::all_rustc();
                assert!(
                    Emitter::default()
                        .add_instructions(&rustc)?
                        .emit_to(&mut stdout_buf)
                        .is_ok()
                );
                let output = String::from_utf8_lossy(&stdout_buf);
                assert!(output.contains("cargo:rustc-env=VERGEN_RUSTC_CHANNEL=this is a bad date"));
                Ok(())
            }();
            assert!(result.is_ok());
        });
    }

    #[test]
    #[serial]
    fn rustc_commit_date_override_works() {
        with_var(
            "VERGEN_RUSTC_COMMIT_DATE",
            Some("this is a bad date"),
            || {
                let result =
                    || -> Result<()> {
                        let mut stdout_buf = vec![];
                        let rustc = Rustc::all_rustc();
                        assert!(
                            Emitter::default()
                                .add_instructions(&rustc)?
                                .emit_to(&mut stdout_buf)
                                .is_ok()
                        );
                        let output = String::from_utf8_lossy(&stdout_buf);
                        assert!(output.contains(
                            "cargo:rustc-env=VERGEN_RUSTC_COMMIT_DATE=this is a bad date"
                        ));
                        Ok(())
                    }();
                assert!(result.is_ok());
            },
        );
    }

    #[test]
    #[serial]
    fn rustc_commit_hash_override_works() {
        with_var(
            "VERGEN_RUSTC_COMMIT_HASH",
            Some("this is a bad date"),
            || {
                let result =
                    || -> Result<()> {
                        let mut stdout_buf = vec![];
                        let rustc = Rustc::all_rustc();
                        assert!(
                            Emitter::default()
                                .add_instructions(&rustc)?
                                .emit_to(&mut stdout_buf)
                                .is_ok()
                        );
                        let output = String::from_utf8_lossy(&stdout_buf);
                        assert!(output.contains(
                            "cargo:rustc-env=VERGEN_RUSTC_COMMIT_HASH=this is a bad date"
                        ));
                        Ok(())
                    }();
                assert!(result.is_ok());
            },
        );
    }

    #[test]
    #[serial]
    fn rustc_host_triple_override_works() {
        with_var(
            "VERGEN_RUSTC_HOST_TRIPLE",
            Some("this is a bad date"),
            || {
                let result =
                    || -> Result<()> {
                        let mut stdout_buf = vec![];
                        let rustc = Rustc::all_rustc();
                        assert!(
                            Emitter::default()
                                .add_instructions(&rustc)?
                                .emit_to(&mut stdout_buf)
                                .is_ok()
                        );
                        let output = String::from_utf8_lossy(&stdout_buf);
                        assert!(output.contains(
                            "cargo:rustc-env=VERGEN_RUSTC_HOST_TRIPLE=this is a bad date"
                        ));
                        Ok(())
                    }();
                assert!(result.is_ok());
            },
        );
    }

    #[test]
    #[serial]
    fn rustc_llvm_version_override_works() {
        with_var(
            "VERGEN_RUSTC_LLVM_VERSION",
            Some("this is a bad date"),
            || {
                let result =
                    || -> Result<()> {
                        let mut stdout_buf = vec![];
                        let rustc = Rustc::all_rustc();
                        assert!(
                            Emitter::default()
                                .add_instructions(&rustc)?
                                .emit_to(&mut stdout_buf)
                                .is_ok()
                        );
                        let output = String::from_utf8_lossy(&stdout_buf);
                        assert!(output.contains(
                            "cargo:rustc-env=VERGEN_RUSTC_LLVM_VERSION=this is a bad date"
                        ));
                        Ok(())
                    }();
                assert!(result.is_ok());
            },
        );
    }

    #[test]
    #[serial]
    fn rustc_semver_override_works() {
        with_var("VERGEN_RUSTC_SEMVER", Some("this is a bad date"), || {
            let result = || -> Result<()> {
                let mut stdout_buf = vec![];
                let rustc = Rustc::all_rustc();
                assert!(
                    Emitter::default()
                        .add_instructions(&rustc)?
                        .emit_to(&mut stdout_buf)
                        .is_ok()
                );
                let output = String::from_utf8_lossy(&stdout_buf);
                assert!(output.contains("cargo:rustc-env=VERGEN_RUSTC_SEMVER=this is a bad date"));
                Ok(())
            }();
            assert!(result.is_ok());
        });
    }
}
