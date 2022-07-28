#!/usr/bin/env bash

set -eu -o pipefail

script_dir="$(dirname -- "${BASH_SOURCE[0]}")"

cd "$script_dir/.."
script_dir="$(pwd)" # replacing it with full path for later

set -x

cargo build --release

rm -rf "$CARGO_TARGET_DIR/mpvserve_dist"
mkdir "$CARGO_TARGET_DIR/mpvserve_dist"

cp -r public "$CARGO_TARGET_DIR/mpvserve_dist/"
cp -r templates "$CARGO_TARGET_DIR/mpvserve_dist/"
cp Rocket.toml "$CARGO_TARGET_DIR/mpvserve_dist/"

cp "$CARGO_TARGET_DIR/release/mpvserve" "$CARGO_TARGET_DIR/mpvserve_dist/"

cd "$CARGO_TARGET_DIR"
tar -cf mpvserve.tar.gz mpvserve_dist
cd "$script_dir"

mv "$CARGO_TARGET_DIR/mpvserve.tar.gz" .
