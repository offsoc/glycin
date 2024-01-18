#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_possible_wrap)]

//! Utilities for building glycin decoders

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod dbus;
pub mod error;
#[cfg(feature = "image-rs")]
pub mod image_rs;
#[cfg(feature = "loader-utils")]
pub mod instruction_handler;
pub mod save_conversion;
#[cfg(feature = "loader-utils")]
pub mod shared_memory;

pub use dbus::*;
pub use error::*;
#[cfg(feature = "loader-utils")]
pub use instruction_handler::*;
pub use save_conversion::*;
#[cfg(feature = "loader-utils")]
pub use shared_memory::*;
