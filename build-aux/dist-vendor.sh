#!/bin/sh -ex

cd "$MESON_PROJECT_DIST_ROOT"

# Remove crates.io packaged part
sed -i 's/"glycin",\?//' Cargo.toml
rm -r glycin
sed -i 's/"glycin-utils",\?//' Cargo.toml
rm -r glycin-utils

cat Cargo.toml

# Use crates.io libraries
VERSION="$($MESON_PROJECT_SOURCE_ROOT/build-aux/crates-version.py)"
for toml in $(find loaders/ -name Cargo.toml); do
  sed -i "s/path = \"..\/..\/glycin-utils\/\"/version = \"$VERSION\"/g" "$toml"
done

sed -i "s/path = \"..\/glycin\/\"/version = \"$VERSION\"/g" tests/Cargo.toml

# Vendor crates.io dependencies
mkdir .cargo
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > .cargo/config
