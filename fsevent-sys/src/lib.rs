#![cfg(target_os = "macos")]
#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

mod fsevent;
pub use fsevent::*;
