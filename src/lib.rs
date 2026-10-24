// Copyright (c) 2022 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! # smolgen - Emit cargo instructions from a build script
//!
//! `smolgen` is a single-crate, minimal-dependency fork of [`vergen`](https://github.com/rustyhorde/vergen).
//! The `vergen`, `vergen-lib`, `vergen-gitcl`, `vergen-git2`, `vergen-gix` and `vergen-pretty` crates
//! are modules of this crate, enabled through features.
//!
//! `smolgen`, when used in conjunction with cargo [build scripts] can emit the following:
//!
//! - Will emit [`cargo:rustc-env=VAR=VALUE`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-envvarvalue)
//!   for each feature you have enabled.  These can be referenced with the [`env`!](std::env!) or [`option_env`!](std::option_env!) macro in your code.
//! - Can emit [`cargo:warning`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargo-warning) outputs if the
//!   [`fail_on_error`](Emitter::fail_on_error) feature is not enabled and the requested variable is defaulted through error or
//!   the [`idempotent`](Emitter::idempotent) flag.
//! - Will emit [`cargo:rerun-if-changed=.git/HEAD`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
//!   and [`cargo:rerun-if-changed=.git/<path_to_ref>`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
//!   if git instructions are emitted.  This is done to ensure any git instructions are regenerated when commits are made.
//! - Will emit [`cargo:rerun-if-changed=build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
//!   to rerun instruction emission if the `build.rs` file changed.
//! - Will emit [`cargo:rerun-if-env-changed=VERGEN_IDEMPOTENT`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
//!   to rerun instruction emission if the `VERGEN_IDEMPOTENT` environment variable has changed.
//! - Will emit [`cargo:rerun-if-env-changed=VERGEN_DEFAULT_ON_ERROR`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
//!   to rerun instruction emission if the `VERGEN_DEFAULT_ON_ERROR` environment variable has changed.
//! - Will emit [`cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
//!   to rerun instruction emission if the `SOURCE_DATE_EPOCH` environment variable has changed.
//! - Will emit custom instructions via the [`AddCustomEntries`] and the [`add_custom_instructions`](Emitter::add_custom_instructions) function.
//!
//! ## Usage
//!
//! 1. Ensure you have build scripts enabled via the `build` configuration in your `Cargo.toml`
//!
//! ```toml
//! [package]
//! #..
//! build = "build.rs"
//! ```
//!
//! 2. Add `smolgen` as a build dependency in `Cargo.toml`, specifying the features you wish to enable.
//!
//! ```toml
//! [dependencies]
//! #..
//!
//! [build-dependencies]
//! # All features enabled
//! smolgen = { version = "0.1", features = ["build", "cargo", "gitcl", "rustc", "si"] }
//! # or
//! smolgen = { version = "0.1", features = ["build"] }
//! # if you wish to disable certain features
//! ```
//!
//! 3. Create a `build.rs` file that uses `smolgen` to emit cargo instructions.  Configuration
//!    starts with [`Emitter`].  Eventually you will call [`emit`](Emitter::emit) to output the
//!    cargo instructions. See the [`emit`](Emitter::emit) documentation for more robust examples.
//!
//! #### Generate all output
//!
//! ```
//! # use smolgen::Result;
//! # use smolgen::Emitter;
#![cfg_attr(feature = "build", doc = r"# use smolgen::Build;")]
#![cfg_attr(feature = "cargo", doc = r"# use smolgen::Cargo;")]
#![cfg_attr(feature = "gitcl", doc = r"# use smolgen::Gitcl;")]
#![cfg_attr(feature = "git2", doc = r"# use smolgen::Git2;")]
#![cfg_attr(feature = "gix", doc = r"# use smolgen::Gix;")]
#![cfg_attr(feature = "rustc", doc = r"# use smolgen::Rustc;")]
#![cfg_attr(feature = "si", doc = r"# use smolgen::Sysinfo;")]
//! #
//! # pub fn main() -> Result<()> {
#![cfg_attr(
    feature = "cargo",
    doc = r#"# let result = temp_env::with_vars([("DEBUG", Some("true")), ("OPT_LEVEL", Some("1")), ("TARGET", Some("x86_64-unknown-linux-gnu"))], || -> Result<()> {"#
)]
//! // NOTE: This will output everything, and requires all features enabled.
//! // NOTE: See the specific builder documentation for configuration options.
#![cfg_attr(feature = "build", doc = r"let build = Build::all_build();")]
#![cfg_attr(feature = "cargo", doc = r"let cargo = Cargo::all_cargo();")]
#![cfg_attr(feature = "gitcl", doc = r"let gitcl = Gitcl::all_git();")]
#![cfg_attr(feature = "git2", doc = r"let git2 = Git2::all_git();")]
#![cfg_attr(feature = "gix", doc = r"let gix = Gix::all_git();")]
#![cfg_attr(feature = "rustc", doc = r"let rustc = Rustc::all_rustc();")]
#![cfg_attr(feature = "si", doc = r"let si = Sysinfo::all_sysinfo();")]
//!
//! Emitter::default()
#![cfg_attr(feature = "build", doc = r"    .add_instructions(&build)?")]
#![cfg_attr(feature = "cargo", doc = r"    .add_instructions(&cargo)?")]
#![cfg_attr(feature = "gitcl", doc = r"    .add_instructions(&gitcl)?")]
#![cfg_attr(feature = "git2", doc = r"    .add_instructions(&git2)?")]
#![cfg_attr(feature = "gix", doc = r"    .add_instructions(&gix)?")]
#![cfg_attr(feature = "rustc", doc = r"    .add_instructions(&rustc)?")]
#![cfg_attr(feature = "si", doc = r"    .add_instructions(&si)?")]
//!     .emit()?;
#![cfg_attr(
    feature = "cargo",
    doc = r"
# Ok(())
# });
# assert!(result.is_ok());"
)]
//! #    Ok(())
//! # }
//! ```
//! #### Sample Output
//! ```text
//! cargo:rustc-env=VERGEN_BUILD_DATE=2024-01-31
//! cargo:rustc-env=VERGEN_BUILD_TIMESTAMP=2024-01-31T03:26:34.065893658Z
//! cargo:rustc-env=VERGEN_CARGO_DEBUG=true
//! cargo:rustc-env=VERGEN_CARGO_FEATURES=
//! cargo:rustc-env=VERGEN_CARGO_OPT_LEVEL=0
//! cargo:rustc-env=VERGEN_CARGO_TARGET_TRIPLE=x86_64-unknown-linux-gnu
//! cargo:rustc-env=VERGEN_CARGO_DEPENDENCIES=anyhow 1.0.79,vergen-pretty 0.3.2
//! cargo:rustc-env=VERGEN_GIT_BRANCH=master
//! cargo:rustc-env=VERGEN_GIT_COMMIT_AUTHOR_EMAIL=emitter@vergen.com
//! cargo:rustc-env=VERGEN_GIT_COMMIT_AUTHOR_NAME=Jason Ozias
//! cargo:rustc-env=VERGEN_GIT_COMMIT_COUNT=44
//! cargo:rustc-env=VERGEN_GIT_COMMIT_DATE=2024-01-30
//! cargo:rustc-env=VERGEN_GIT_COMMIT_MESSAGE=depsup
//! cargo:rustc-env=VERGEN_GIT_COMMIT_TIMESTAMP=2024-01-30T21:43:43.000000000Z
//! cargo:rustc-env=VERGEN_GIT_DESCRIBE=0.1.0-beta.1-15-g728e25c
//! cargo:rustc-env=VERGEN_GIT_DIRTY=false
//! cargo:rustc-env=VERGEN_GIT_SHA=728e25ca5bb7edbbc505f12b28c66b2b27883cf1
//! cargo:rustc-env=VERGEN_RUSTC_CHANNEL=nightly
//! cargo:rustc-env=VERGEN_RUSTC_COMMIT_DATE=2024-01-29
//! cargo:rustc-env=VERGEN_RUSTC_COMMIT_HASH=5518eaa946291f00471af8b254b2a1715f234882
//! cargo:rustc-env=VERGEN_RUSTC_HOST_TRIPLE=x86_64-unknown-linux-gnu
//! cargo:rustc-env=VERGEN_RUSTC_LLVM_VERSION=17.0
//! cargo:rustc-env=VERGEN_RUSTC_SEMVER=1.77.0-nightly
//! cargo:rustc-env=VERGEN_SYSINFO_NAME=Arch Linux
//! cargo:rustc-env=VERGEN_SYSINFO_OS_VERSION=Linux  Arch Linux
//! cargo:rustc-env=VERGEN_SYSINFO_USER=jozias
//! cargo:rustc-env=VERGEN_SYSINFO_TOTAL_MEMORY=31 GiB
//! cargo:rustc-env=VERGEN_SYSINFO_CPU_VENDOR=AuthenticAMD
//! cargo:rustc-env=VERGEN_SYSINFO_CPU_CORE_COUNT=8
//! cargo:rustc-env=VERGEN_SYSINFO_CPU_NAME=cpu0,cpu1,cpu2,cpu3,cpu4,cpu5,cpu6,cpu7
//! cargo:rustc-env=VERGEN_SYSINFO_CPU_BRAND=AMD Ryzen Threadripper 1900X 8-Core Processor
//! cargo:rustc-env=VERGEN_SYSINFO_CPU_FREQUENCY=3792
//! cargo:rerun-if-changed=/home/jozias/projects/rust-lang/vergen-cl/.git/HEAD
//! cargo:rerun-if-changed=/home/jozias/projects/rust-lang/vergen-cl/.git/refs/heads/master
//! cargo:rerun-if-changed=build.rs
//! cargo:rerun-if-env-changed=VERGEN_IDEMPOTENT
//! cargo:rerun-if-env-changed=VERGEN_DEFAULT_ON_ERROR
//! cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH
//! ```
//!
//! #### Generate specific output
//!
//! ```
//! # use smolgen::Result;
//! # use smolgen::Emitter;
#![cfg_attr(feature = "build", doc = r"# use smolgen::Build;")]
#![cfg_attr(feature = "cargo", doc = r"# use smolgen::Cargo;")]
#![cfg_attr(feature = "gitcl", doc = r"# use smolgen::Gitcl;")]
#![cfg_attr(feature = "git2", doc = r"# use smolgen::Git2;")]
#![cfg_attr(feature = "gix", doc = r"# use smolgen::Gix;")]
#![cfg_attr(feature = "rustc", doc = r"# use smolgen::Rustc;")]
#![cfg_attr(feature = "si", doc = r"# use smolgen::Sysinfo;")]
//! #
//! # pub fn main() -> Result<()> {
#![cfg_attr(
    feature = "cargo",
    doc = r#"# let result = temp_env::with_vars([("DEBUG", Some("true")), ("OPT_LEVEL", Some("1")), ("TARGET", Some("x86_64-unknown-linux-gnu"))], || -> Result<()> {"#
)]
//! // NOTE: This will output only the instructions specified.
//! // NOTE: See the specific builder documentation for configuration options.
#![cfg_attr(
    feature = "build",
    doc = r"let build = Build::builder().build_timestamp(true).build();"
)]
#![cfg_attr(
    feature = "cargo",
    doc = r"let cargo = Cargo::builder().opt_level(true).build();"
)]
#![cfg_attr(
    feature = "gitcl",
    doc = r"let gitcl = Gitcl::builder().commit_timestamp(true).build();"
)]
#![cfg_attr(
    feature = "git2",
    doc = r"let git2 = Git2::builder().commit_timestamp(true).build();"
)]
#![cfg_attr(
    feature = "gix",
    doc = r"let gix = Gix::builder().commit_timestamp(true).build();"
)]
#![cfg_attr(
    feature = "rustc",
    doc = r"let rustc = Rustc::builder().semver(true).build();"
)]
#![cfg_attr(
    feature = "si",
    doc = r"let si = Sysinfo::builder().cpu_core_count(true).build();"
)]
//!
//! Emitter::default()
#![cfg_attr(feature = "build", doc = r"    .add_instructions(&build)?")]
#![cfg_attr(feature = "cargo", doc = r"    .add_instructions(&cargo)?")]
#![cfg_attr(feature = "gitcl", doc = r"    .add_instructions(&gitcl)?")]
#![cfg_attr(feature = "git2", doc = r"    .add_instructions(&git2)?")]
#![cfg_attr(feature = "gix", doc = r"    .add_instructions(&gix)?")]
#![cfg_attr(feature = "rustc", doc = r"    .add_instructions(&rustc)?")]
#![cfg_attr(feature = "si", doc = r"    .add_instructions(&si)?")]
//!     .emit()?;
#![cfg_attr(
    feature = "cargo",
    doc = r"
#   Ok(())
# });
#    assert!(result.is_ok());"
)]
//! #   Ok(())
//! # }
//! ```
//! #### Sample Output
//! ```text
//! cargo:rustc-env=VERGEN_BUILD_TIMESTAMP=2024-01-31T03:26:34.065893658Z
//! cargo:rustc-env=VERGEN_CARGO_OPT_LEVEL=0
//! cargo:rustc-env=VERGEN_GIT_COMMIT_TIMESTAMP=2024-01-30T21:43:43.000000000Z
//! cargo:rustc-env=VERGEN_RUSTC_SEMVER=1.77.0-nightly
//! cargo:rustc-env=VERGEN_SYSINFO_CPU_CORE_COUNT=8
//! cargo:rerun-if-changed=/home/jozias/projects/rust-lang/vergen-cl/.git/HEAD
//! cargo:rerun-if-changed=/home/jozias/projects/rust-lang/vergen-cl/.git/refs/heads/master
//! cargo:rerun-if-changed=build.rs
//! cargo:rerun-if-env-changed=VERGEN_IDEMPOTENT
//! cargo:rerun-if-env-changed=VERGEN_DEFAULT_ON_ERROR
//! cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH
//! ```
//!
//! 4. Use the [`env!`](std::env!) or [`option_env!`](std::option_env!) macro in your code to read the environment variables.
//!
//! ```
//! if let Some(timestamp) = option_env!("VERGEN_BUILD_TIMESTAMP") {
//!     println!("Build Timestamp: {timestamp}");
//! }
//! if let Some(describe) = option_env!("VERGEN_GIT_DESCRIBE") {
//!     println!("git describe: {describe}");
//! }
//! ```
//!
//! ## Features
//! No features are enabled by default. You **must** specifically enable the features you wish to use.
//!
//! |    Feature     | Enables                                                                  |
//! | -------------- | ------------------------------------------------------------------------ |
//! |     build      | `VERGEN_BUILD_*` instructions                                            |
//! |     cargo      | `VERGEN_CARGO_*` instructions                                            |
//! | cargo_metadata | `VERGEN_CARGO_DEPENDENCIES` (uses `cargo_metadata`)                      |
//! |     gitcl      | `VERGEN_GIT_*` instructions using the `git` binary                       |
//! |      git2      | `VERGEN_GIT_*` instructions using the `git2` library                     |
//! |      gix       | `VERGEN_GIT_*` instructions using the `gix` library                      |
//! |     rustc      | `VERGEN_RUSTC_*` instructions                                            |
//! |       si       | `VERGEN_SYSINFO_*` instructions (uses `sysinfo`)                         |
//! |  emit_and_set  | The [`emit_and_set`](Emitter::emit_and_set) function                     |
//! |     pretty     | The [`pretty`] module, which formats the emitted environment variables  |
//!
//! ## Environment Variables
//! `smolgen` currently recognizes the following environment variables. The full list of the environment variable names can be
//! found in [`constants`].
//!
//! | Variable | Functionality |
//! | -------- | ------------- |
//! | `VERGEN_IDEMPOTENT` | If this environment variable is set `smolgen` will **always** use the idempotent output feature regardless of the configuration set in `build.rs`, even when the requested values could be generated normally.  This exists mainly to allow package maintainers to force idempotent output to generate deterministic binary output. |
//! | `VERGEN_DEFAULT_ON_ERROR` | If this environment variable is set `smolgen` will emit the idempotent default value for any instruction it cannot otherwise generate (e.g. `VERGEN_GIT_*` when building outside a git worktree), rather than leaving the variable unset.  Unlike `VERGEN_IDEMPOTENT`, values that *can* be generated are still emitted normally. |
//! | `SOURCE_DATE_EPOCH` | If this environment variable is set `smolgen` will use the value (unix time since epoch) as the basis for a time based instructions.  This can help emit deterministic instructions. |
//! | `VERGEN_BUILD_*` | If this environment variable is set `smolgen` will use the value you specify for the output rather than generating it. |
//! | `VERGEN_CARGO_*` | If this environment variable is set `smolgen` will use the value you specify for the output rather than generating it. |
//! | `VERGEN_GIT_*` | If this environment variable is set `smolgen` will use the value you specify for the output rather than generating it. |
//! | `VERGEN_RUSTC_*` | If this environment variable is set `smolgen` will use the value you specify for the output rather than generating it. |
//! | `VERGEN_SYSINFO_*` | If this environment variable is set `smolgen` will use the value you specify for the output rather than generating it. |
//!
//! [build scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script

#![cfg_attr(docsrs, feature(doc_cfg))]

/// A [`Result`](core::result::Result) that defaults to [`Error`].
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Defines a builder that wraps the built type, starting from its [`Default`], with one setter
/// per listed field.
macro_rules! builder {
    (
        $builder:ident => $ty:ident {
            $( $(#[$attr:meta])* $field:ident: $field_ty:ty ),* $(,)?
        }
    ) => {
        #[doc = concat!("A builder for [`", stringify!($ty), "`].")]
        #[derive(Clone, Debug)]
        pub struct $builder($ty);

        impl $ty {
            #[doc = concat!("Creates a builder for [`", stringify!($ty), "`].")]
            #[must_use]
            pub fn builder() -> $builder {
                $builder(<$ty>::default())
            }
        }

        impl $builder {
            $(
                $(#[$attr])*
                #[must_use]
                pub fn $field(mut self, $field: $field_ty) -> Self {
                    self.0.$field = $field;
                    self
                }
            )*

            #[doc = concat!("Builds the [`", stringify!($ty), "`].")]
            #[must_use]
            pub fn build(self) -> $ty {
                self.0
            }
        }
    };
}

mod config;
pub mod constants;
mod emitter;
mod entries;
mod error;
mod keys;
mod utils;

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "cargo")]
mod cargo;
#[cfg(any(
    feature = "build",
    feature = "gitcl",
    feature = "git2",
    feature = "gix"
))]
mod datetime;
#[cfg(feature = "git2")]
mod git2;
#[cfg(feature = "gitcl")]
mod gitcl;
#[cfg(feature = "gix")]
mod gix;
#[cfg(feature = "pretty")]
pub mod pretty;
#[cfg(feature = "rustc")]
mod rustc;
#[cfg(feature = "si")]
mod si;

#[cfg(test)]
extern crate self as smolgen;
#[cfg(test)]
#[path = "../tests/test_util/mod.rs"]
mod test_util;

pub use self::config::{Describe, DescribeBuilder, Dirty, DirtyBuilder, Sha, ShaBuilder};
pub use self::emitter::Emitter;
#[doc(hidden)]
pub use self::entries::test_gen::{CustomInsGen, CustomInsGenBuilder};
pub use self::entries::{
    Add as AddEntries, AddCustom as AddCustomEntries, CargoRerunIfChanged, CargoRustcEnvMap,
    CargoWarning, DefaultConfig,
};
pub use self::error::Error;
pub use self::keys::vergen_key::VergenKey;
#[cfg(feature = "vcs_info")]
#[doc(hidden)]
pub use self::utils::vcs_info;
#[doc(hidden)]
pub use self::utils::{add_default_map_entry, add_map_entry, count_idempotent};

#[cfg(feature = "build")]
pub use self::build::{Build, BuildBuilder};
#[cfg(feature = "cargo")]
pub use self::cargo::{Cargo, CargoBuilder};
#[cfg(feature = "git2")]
pub use self::git2::{Git2, Git2Builder};
#[cfg(feature = "gitcl")]
pub use self::gitcl::{Gitcl, GitclBuilder};
#[cfg(feature = "gix")]
pub use self::gix::{Gix, GixBuilder};
#[cfg(feature = "rustc")]
pub use self::rustc::{Rustc, RustcBuilder};
#[cfg(feature = "si")]
pub use self::si::{Sysinfo, SysinfoBuilder};
#[cfg(feature = "cargo_metadata")]
pub use cargo_metadata::DependencyKind;
#[cfg(feature = "si")]
pub use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind};
