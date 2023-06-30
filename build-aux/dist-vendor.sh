#!/bin/sh -e

cd "$MESON_PROJECT_DIST_ROOT"


# Remove crates.io packaged part
sed -i '/"glycin"/d' Cargo.toml
rm -r glycin
sed -i '/"glycin-utils"/d' Cargo.toml
rm -r glycin-utils

# Use crates.io libraries
VERSION="$($MESON_PROJECT_SOURCE_ROOT/build-aux/crates-version.py)"
for toml in $(find loaders/ -name Cargo.toml); do
  sed -i "s/path = \"..\/..\/glycin-utils\/\"/version = \"$VERSION\"/g" "$toml"
done

# Vendor crates.io dependencies
mkdir .cargo
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > .cargo/config
