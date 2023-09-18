#!/bin/sh -ex

cd "$MESON_PROJECT_DIST_ROOT"

# Remove crates.io packaged part
sed -i 's/"glycin",\?//' Cargo.toml
rm -r glycin
awk -i inplace -v RS= -v ORS='\n\n' '!/name = "glycin"/' Cargo.lock

sed -i 's/"glycin-utils",\?//' Cargo.toml
rm -r glycin-utils
awk -i inplace -v RS= -v ORS='\n\n' '!/name = "glycin-utils"/' Cargo.lock

cat Cargo.toml

# Use crates.io libraries
VERSION="$($MESON_PROJECT_SOURCE_ROOT/build-aux/crates-version.py)"
for toml in $(find loaders/ -name Cargo.toml); do
  sed -i "s/path = \"..\/..\/glycin-utils\/\"/version = \"$VERSION\"/g" "$toml"
done

sed -i "s/path = \"..\/glycin\/\"/version = \"$VERSION\"/g" tests/Cargo.toml

# Trigger update in Cargo.lock
for toml in $(find loaders/ -name Cargo.toml); do
  cargo add --manifest-path="$toml" glycin-utils@$VERSION
done

cargo add --manifest-path=tests/Cargo.toml glycin@$VERSION

# Vendor crates.io dependencies
mkdir .cargo
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > .cargo/config
