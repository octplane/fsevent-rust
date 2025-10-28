#![cfg(target_os = "macos")]
#![deprecated = "deprecated in favour of the `objc2-core-services` crate"]

mod fsevent;
pub use fsevent::*;
