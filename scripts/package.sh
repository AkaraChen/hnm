#!/bin/sh
set -eu
# Called on each native runner after cargo build --release --locked --target.
target=${1:?Rust target required}
version=$(awk -F '"' '/^version = / {print $2; exit}' Cargo.toml)
asset=hnm-v$version-$target.tar.gz
mkdir -p dist
tar -czf "dist/$asset" -C "target/$target/release" hnm
(cd dist && if command -v sha256sum >/dev/null 2>&1; then sha256sum "$asset"; else shasum -a 256 "$asset"; fi) > "dist/$asset.sha256"
