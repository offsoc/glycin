# Contributing to Glycin

Running tests

```sh
$ meson setup -Dprofile=dev builddir
$ meson test -vC builddir
```

## Useful Commands

Glycin comes with a few tools in `tools/` that can be especially helpful for development. You can force glycin to use the loaders that you have built by using `GLYCIN_DATA_DIR`.

```sh
# Configure to install glycin loaders into `install/` directory
$ meson setup -Dprofile=dev --prefix=$(pwd)/install builddir
$ meson install -C builddir
# Force glycin to use the previously built loaders
$ GLYCIN_DATA_DIR=$(pwd)/install/share cargo r -p tools --bin glycin-image-info image.png
```

Get Exif information

```sh
$ identify -format '%[EXIF:*]' <image>
```

Increase memory usage (this can crash your PC)

```sh
$ sudo swapoff -a
$ stress-ng --vm-bytes 20G --vm-keep -m 1
```

## Test D-Bus API stability

The following test will ensure that the lastest API documented in `docs/` hasn't changed.

```
$ GLYCIN_DATA_DIR=$(pwd)/install/share cargo test -p tests -- dbus_api_stability --include-ignored
```

## Resources

- [xdg/shared-mime-info](https://gitlab.freedesktop.org/xdg/shared-mime-info/-/blob/master/data/freedesktop.org.xml.in)