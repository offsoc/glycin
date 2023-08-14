# Contributing to Glycin

Running tests

```sh
$ meson setup -Dprofile=dev --prefix "$(pwd)/output" builddir
$ meson install -C builddir && meson test -vC builddir
```