// Copyright (c) 2022 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Test utilities shared by the unit and integration tests.

#![allow(dead_code, unreachable_pub, unused_imports)]

mod repo;
mod utils;

pub use self::repo::{RUNNER_TEMP_ENV, TEST_MTIME, TestRepos};
pub use self::utils::{with_cargo_vars, with_cargo_vars_ext};
