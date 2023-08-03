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
exec = /usr/libexec/glycin/0+/glycin-image-rs
```

Where the part behind `loader` is a mime-type and the value of `exec` can be any executable path.

### Existing compatibility versions

Not every new major version of the library has to break compatibility with the loaders. The following compatibility versions currently exist

| compat-version |
|----------------|
| 0+ |

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
| JPEG 2000 | TODO     | ✘   | —    | ✘    | ？   | ✘         | jpeg2k? + openjpeg (C)     |
| JPEG XL   | jxl      | #7  | ✘    | ✘    | ？   | ✘         | jxl-oxide                  |
| OpenEXR   | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| PNG       | image-rs | ✔   | ✘    | ✔    | ✘   | ✔         | image-rs                   |
| PNM       | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| SVG       | image-rs | ✘   | —    | —    | ✘ * | —         | librsvg + gdk-pixbuf       |
| TGA       | image-rs | —   | —    | —    | —   | —         | image-rs                   |
| TIFF      | image-rs | ✔   | —    | ✔    | ✘   | —         | image-rs                   |
| WEBP      | image-rs | ✔   | —    | ✔    | ✘   | ✔         | image-rs + libwebp (C)     |

| Symbol | Meaning                                        |
|--------|------------------------------------------------|
| ✔      | Supported                                      |
| ✘      | Supported by format but not implemented yet    |
| —      | Not available for this format                  |
| ？      | Unclear if supported by format, needs research |
| *      | Unclear if used in practice, needs research    |
