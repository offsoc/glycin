# Glycin

Glycin allows to decode images into [`gdk::Texture`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk4/struct.Texture.html)s and to extract image metadata.
The decoding happens in sandboxed modular *image loaders*.

- [glycin](glycin) – The image library
- [glycin-utils](glycin-utils) – Utilities to write loaders for glycin
- [loaders](loaders) – Glycin loaders for several formats

## Example

```rust
let file = gio::File::for_path("image.jpg");
let image = Loader::new(file).request().await?;

let height = image.info().height;
let texture = image.next_frame().await?.texture;
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

| Format   | Decoder  | ICC | CICP | EXIF | XMP | Animation | Library                    |
|----------|----------|-----|------|------|-----|-----------|----------------------------|
| AVIF     | heif     | ✔   | ✔    | ✔    | ✘   | ✘         | libheif-rs + libheif (C++) |
| BMP      | image-rs | ✘   | —    | —    | —   | —         | image-rs                   |
| DDS      | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| farbfeld | no mime  | —   | —    | —    | —   | —         | image-rs                   |
| QOI      | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| GIF      | image-rs | ✘   | —    | —    | ✘   | ✔         | image-rs                   |
| HEIC     | heif     | ✔   | ✔    | ✔    | ✘   | ✘         | libheif-rs + libheif (C++) |
| ICO      | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| JPEG     | image-rs | ✔   | —    | ✔    | ✘   | —         | image-rs                   |
| JPEG XL  | jxl      | ✔   | ✘    | ✔    | ✘   | ✘         | jpegxl-rs + libjxl (C++)   |
| OpenEXR  | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| PNG      | image-rs | ✔   | ✘    | ✔    | ✘   | ✔         | image-rs                   |
| PNM      | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| SVG      | image-rs | ✘   | —    | —    | ✘   | —         | librsvg + gdk-pixbuf (C)   |
| TGA      | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| TIFF     | image-rs | ✔   | —    | ✔    | ✘   | —         | image-rs                   |
| WEBP     | image-rs | ✔   | —    | ✔    | ✘   | ✔         | image-rs                   |

| Symbol | Meaning                                     |
|--------|---------------------------------------------|
| ✔      | Supported                                   |
| ✘      | Supported by format but not implemented yet |
| —      | Not available for this format               |

## Sandboxing and Inner Workings

Glycin spawns one process per image file. The communication between glycin and the loader takes place via peer-to-peer D-Bus over a Unix socket.

Glycin supports a sandbox mechanism inside and outside of Flatpaks. Outside of Flatpaks, the following mechanisms are used: The image loader binary is spawned via `bwrap`. The bubblewrap configuration only allows for minimal interaction with the host system. Only necessary parts of the filesystem are mounted and only with read access. There is no direct network access. Environment variables are not passed to the sandbox. Before forking the process the memory usage is limited via calling `setrlimit` and syscalls are limited to an allow-list via seccomp filters.

Inside of Flatpaks the `flatpak-spawn --sandbox` command is used. This restricts the access to the filesystem in a similar way as the direct `bwrap` call. The memory usage is currently not limited. No additional seccomp filters are applied to the existing Flatpak seccomp rules.

The GFile content is streamed to the loader via a Unix socket. This way, loaders can load contents that require network access, without having direct network access themselves. Formats like SVG set the `ExposeBaseDir = true` option in their config. This option causes the original image file's directory to be mounted into the sandbox to include external image files from there. The `ExposeBaseDir` option has no effect for `flatpak-spawn` sandboxes since they don't support this feature.

The loaders provide the texture data via a memfd that is sealed by glycin and then given as an mmap to GDK. For animations and SVGs the sandboxed process is kept alive for new frames or tiles as long as needed.

For information on how to implement a loader, please consult the [`glycin-utils` docs](https://docs.rs/glycin-utils/).


## Building and Testing

- The `-Dloaders` option allows to only build certain loaders.
- The `-Dtest_skip_ext` option allows to skip certain image filename extensions during tests. The `-Dtest_skip_ext=heic` might be needed if x265 is not available.
- Running integration tests requires the glycin loaders to be installed. By default this is done by `meson test` automatically. This behavior can be changed by setting `-Dtest_skip_install=true`.
- The `glycin` crate has an example, `glycin-render` that will load the image passed as a parameter and render it as a PNG into `output.png` in the current directory.

### Packaging

Distributions need to package the loader binaries and their configs independent of apps. The loaders build and installed via meson.

Apps will depend on the `glycin` crate to make use of the installed binary loaders.

[![Packaging Status](https://repology.org/badge/vertical-allrepos/glycin-loaders.svg?exclude_unsupported=1&header=)](https://repology.org/project/glycin-loaders/versions)

## License

SPDX-License-Identifier: MPL-2.0 OR LGPL-2.1-or-later
