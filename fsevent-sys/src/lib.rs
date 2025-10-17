#![cfg(target_os = "macos")]

pub mod core_foundation;
mod fsevent;

pub use fsevent::*;
