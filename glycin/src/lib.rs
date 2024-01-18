#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_possible_wrap)]

//! Glycin allows to decode images into [`gdk::Texture`]s and to extract image
//! metadata. The decoding happens in sandboxed modular image loaders that have
//! to be provided as binaries. The [`glycin-utils`](glycin_utils) for more
//! details.
//!
//! # Example
//!
//! ```no_run
//! # use glycin::*;
//! # async_global_executor::block_on(async {
//! let file = gio::File::for_path("image.jpg");
//! let image = ImageRequest::new(file).request().await?;
//!
//! let height = image.info().height;
//! let frame = image.next_frame().await?;
//! # Ok::<(), Error>(()) });
//! ```
//!
//! You can pass the [`texture`](Frame#structfield.texture) of a [`Frame`] to
//! [`gtk4::Image::from_paintable()`] to display the image.
//!
//! # Features
//!
//! - `tokio` – Makes glycin compatible with [`zbus`] using [`tokio`].
//!
//! [`gtk4::Image::from_paintable()`]: https://gtk-rs.org/gtk4-rs/git/docs/gtk4/struct.Image.html#method.from_paintable

mod api;
mod config;
mod dbus;
mod icc;
mod sandbox;

pub use api::*;
pub use config::COMPAT_VERSION;
pub use glycin_utils::{ImageInfo, ImageInfoDetails, RemoteError};
