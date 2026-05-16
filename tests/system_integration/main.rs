//! System-integration tests.
//!
//! This is a deliberately separate test crate for checks that validate the binding
//! against artefacts shipped by the *installed* MoneyMoney.app on the developer's
//! machine — never against vendored copies. The artefacts (e.g. `MoneyMoney.sdef`)
//! are proprietary and must not enter our commit history.
//!
//! Each test in this crate resolves its inputs at runtime:
//!   1. via an environment variable (so CI or sandboxed setups can override the
//!      path),
//!   2. otherwise via the canonical macOS install location.
//!
//! If neither resolves, the test prints a clear `SKIP:` message and returns Ok.
//! Plain `cargo test` on machines without MoneyMoney therefore stays green; runs
//! with the app installed actually exercise the assertions.
//!
//! Add new system-integration tests as sibling modules: `mod foo;` below, with the
//! implementation in `tests/system_integration/foo.rs`.

mod sdef;
