#!/usr/bin/env bash

set -eu -o pipefail

script_dir="$(dirname -- "${BASH_SOURCE[0]}")"

cd "$script_dir/.."
script_dir="$(pwd)" # replacing it with full path for later

set -x

cargo build --release

rm -rf target/package
mkdir target/package

cp -r public target/package/
cp -r templates target/package/
cp Rocket.toml target/package/

cp target/release/mpvserve target/package/

cd target/package/
tar -cf mpvserve.tar.gz ./*
cd "$script_dir"

mv target/package/mpvserve.tar.gz .
