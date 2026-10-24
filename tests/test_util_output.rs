//! Tests for the shared test utilities, formerly the `test_util` crate.

mod test_util;

use serial_test::serial;
use smolgen::Result;
use std::env::var;
use test_util::{RUNNER_TEMP_ENV, TestRepos, with_cargo_vars, with_cargo_vars_ext};

#[test]
#[serial]
fn temp_dir_works() -> Result<()> {
    temp_env::with_var(RUNNER_TEMP_ENV, None::<String>, || {
        let repo = || -> Result<TestRepos> {
            let repo = TestRepos::new(false, false, false)?;
            Ok(repo)
        }();
        assert!(repo.is_ok());
    });
    Ok(())
}

#[test]
#[serial]
fn test_repos_are_removed_on_drop() -> Result<()> {
    let path = {
        let repo = TestRepos::new(false, false, false)?;
        let path = repo.path();
        assert!(gix::discover(&path).is_ok());
        assert!(path.exists());
        path
    };
    // When dropped, the repositories will be removed.
    assert!(!path.exists());
    Ok(())
}

#[test]
#[serial]
fn test_repos_path_exists() -> Result<()> {
    let repo = TestRepos::new(false, false, false)?;
    assert!(repo.path().exists());
    Ok(())
}

#[test]
#[serial]
fn with_cargo_vars_works() {
    let result = with_cargo_vars(|| {
        assert_eq!(var("OPT_LEVEL")?, "1");
        Ok(())
    });
    assert!(result.is_ok());
}

#[test]
#[serial]
fn with_cargo_vars_ext_works() {
    let result = with_cargo_vars_ext(&[("MY_VAR", Some("12"))], || {
        assert_eq!(var("MY_VAR")?, "12");
        assert_eq!(var("OPT_LEVEL")?, "1");
        Ok(())
    });
    assert!(result.is_ok());
}
