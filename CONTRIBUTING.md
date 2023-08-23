# Contributing to Glycin

Running tests

```sh
$ meson setup -Dprofile=dev --prefix "$(pwd)/output" builddir
$ meson test -vC builddir
```

## Useful Commands

```sh
$ identify -format '%[EXIF:*]' <image>
```

## Resources

- [xdg/shared-mime-info](https://gitlab.freedesktop.org/xdg/shared-mime-info/-/blob/master/data/freedesktop.org.xml.in)