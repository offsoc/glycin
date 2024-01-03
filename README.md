# Glycin

Glycin allows to decode images into [`gdk::Texture`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk4/struct.Texture.html)s and to extract image metadata.
The decoding happens in sandboxed modular *image loaders*.

- [glycin](glycin) – The image library
- [glycin-utils](glycin-utils) – Utilities to write loaders for glycin
- [loaders](loaders) – Glycin loaders for several formats

## Example

```rust
let file = gio::File::for_path("image.jpg");
let image = ImageRequest::new(file).request().await?;

let height = image.info().height;
let texture = image.next_frame().await?;
```

## Image loader configuration

Loader configurations are read from `XDG_DATA_DIRS` and `XDG_DATA_HOME`. The location is typically of the from

```
<data-dir>/share/glycin/<compat-version>+/conf.d/<loader-name>.conf
```

so for example

```
<data-dir>/share/glycin/0+/conf.d/glyicn-image-rs.conf
```

The configs are glib KeyFiles of the the form

```ini
[loader:image/png]
Exec = /usr/libexec/glycin/0+/glycin-image-rs
```

Where the part behind `loader` is a mime-type and the value of `Exec` can be any executable path.

### Existing compatibility versions

Not every new major version of the library has to break compatibility with the loaders. The formal definition is available in [`docs/`](docs/). The following compatibility versions currently exist

| compat-version |
|----------------|
| 0+ |
| 1+ |

## Supported image formats

The following features are supported by the glycin loaders provided in the [loaders](loaders) directory.

| Format    | Decoder  | ICC | CICP | EXIF | XMP | Animation | Library                    |
|-----------|----------|-----|------|------|-----|-----------|----------------------------|
| AVIF      | heif     | ✔   | ✔    | ✔    | ✘   | ✘         | libheif-rs + libheif (C++) |
| BMP       | image-rs | ✘   | —    | —    | —   | —         | image-rs                   |
| DDS       | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| farbfeld  | no mime  | —   | —    | —    | —   | —         | image-rs                   |
| QOI       | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| GIF       | image-rs | ✘ * | —    | —    | ✘   | ✔         | image-rs                   |
| HEIC      | heif     | ✔   | ✔    | ✔    | ✘   | ✘         | libheif-rs + libheif (C++) |
| ICO       | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| JPEG      | image-rs | ✔   | —    | ✔    | ✘   | —         | image-rs                   |
| JPEG XL   | jxl      | ✔   | ✘    | ✘    | ✘   | ✘         | jxl-oxide                  |
| OpenEXR   | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| PNG       | image-rs | ✔   | ✘    | ✔    | ✘   | ✔         | image-rs                   |
| PNM       | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| SVG       | image-rs | ✘   | —    | —    | ✘   | —         | librsvg + gdk-pixbuf       |
| TGA       | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| TIFF      | image-rs | ✔   | —    | ✔    | ✘   | —         | image-rs                   |
| WEBP      | image-rs | ✔   | —    | ✔    | ✘   | ✔         | image-rs                   |

| Symbol | Meaning                                        |
|--------|------------------------------------------------|
| ✔      | Supported                                      |
| ✘      | Supported by format but not implemented yet    |
| —      | Not available for this format                  |
| *      | Unclear if used in practice, needs research    |

## Building and Testing

- The `-Dloaders` option allows to only build certain loaders.
- The `-Dtest_skip_ext` option allows to skip certain image filename extensions during tests. The `-Dtest_skip_ext=heic` might be needed if x265 is not available.
- Running integration tests requires the glycin loaders to be installed. By default this is done by `meson test` automatically. This behavior can be changed by setting `-Dtest_skip_install=true`.
- The `glycin` crate has an example, `glycin-render` that will load the image passed as a parameter and render it as a PNG into `output.png` in the current directory.

### Packaging

Distributions need to package the loader binaries and their configs independent of apps. The loaders build and installed via meson.

Apps will depend on the `glycin` crate to make use of the installed binary loaders.

[![Packaging Status](https://repology.org/badge/vertical-allrepos/glycin-loaders.svg?exclude_unsupported=1&header=)](https://repology.org/project/glycin-loaders/versions)

## Inner Workings

Glycin spawns one sandboxed process per image file via `bwrap` or `flatpak-spawn`. The communication happens via peer-to-peer D-Bus over a UNIX socket. The file data is read from a `GFile` and sent to the sandbox via a separate UNIX socket. The texture data is provided from the sandbox via a memfd that is sealed afterward and given as an mmap to GTK. For animations and SVGs the sandboxed process is kept alive for new frames or tiles as long as needed.

To implement a new loader, please consult the [`glycin-utils` docs](https://docs.rs/glycin-utils/).

## License

SPDX-License-Identifier: MPL-2.0 OR LGPL-2.1-or-later
