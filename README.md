# smolgen

[![CI](https://github.com/alloy-rs/smolgen/actions/workflows/ci.yml/badge.svg)](https://github.com/alloy-rs/smolgen/actions/workflows/ci.yml)

Emit `cargo:rustc-env` instructions from build scripts, for use with the `env!` and `option_env!` macros.

`smolgen` is a fork of [`vergen`](https://github.com/rustyhorde/vergen) that merges `vergen`, `vergen-lib`,
`vergen-gitcl`, `vergen-git2`, `vergen-gix` and `vergen-pretty` into one crate with few dependencies.
Each former crate is a module enabled by a feature, and the emitted variables keep their `VERGEN_*` names.

| Feature          | Enables                                                     |
| ---------------- | ----------------------------------------------------------- |
| `build`          | `VERGEN_BUILD_*` instructions                               |
| `cargo`          | `VERGEN_CARGO_*` instructions                               |
| `cargo_metadata` | `VERGEN_CARGO_DEPENDENCIES` (uses `cargo_metadata`)         |
| `gitcl`          | `VERGEN_GIT_*` instructions using the `git` binary          |
| `git2`           | `VERGEN_GIT_*` instructions using the `git2` library        |
| `gix`            | `VERGEN_GIT_*` instructions using the `gix` library         |
| `rustc`          | `VERGEN_RUSTC_*` instructions                               |
| `si`             | `VERGEN_SYSINFO_*` instructions (uses `sysinfo`)            |
| `emit_and_set`   | `Emitter::emit_and_set`                                     |
| `pretty`         | The `pretty` module, formerly `vergen-pretty`               |

## Usage

```toml
[build-dependencies]
smolgen = { version = "0.1", features = ["build", "cargo", "gitcl", "rustc"] }
```

```rust,ignore
// build.rs
use smolgen::{Build, Cargo, Emitter, Gitcl, Rustc};

fn main() -> smolgen::Result<()> {
    Emitter::default()
        .add_instructions(&Build::all_build())?
        .add_instructions(&Cargo::all_cargo())?
        .add_instructions(&Gitcl::all_git())?
        .add_instructions(&Rustc::all_rustc())?
        .emit()
}
```

```rust,ignore
// main.rs
fn main() {
    println!("git sha: {}", env!("VERGEN_GIT_SHA"));
}
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
