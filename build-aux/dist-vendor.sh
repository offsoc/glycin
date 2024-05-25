#!/bin/sh -ex

cd "$MESON_PROJECT_DIST_ROOT"

# Remove crates.io packaged part
sed -i 's/"glycin",\?//' Cargo.toml
rm -r glycin
awk -i inplace -v RS= -v ORS='\n\n' '!/name = "glycin"/' Cargo.lock

sed -i 's/"glycin-utils",\?//' Cargo.toml
rm -r glycin-utils
awk -i inplace -v RS= -v ORS='\n\n' '!/name = "glycin-utils"/' Cargo.lock

sed -i 's/"tools",\?//' Cargo.toml
rm -r tools
awk -i inplace -v RS= -v ORS='\n\n' '!/name = "tools"/' Cargo.lock

cat Cargo.toml

# Use crates.io libraries
VERSION="$($MESON_PROJECT_SOURCE_ROOT/build-aux/crates-version.py)"

sed -i "s/, path = \"glycin-utils\/\"//g" Cargo.toml
sed -i "s/path = \"glycin\/\"/version = \"$VERSION\"/g" Cargo.toml

cargo update glycin glycin-utils

# Vendor crates.io dependencies
mkdir .cargo
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > .cargo/config
