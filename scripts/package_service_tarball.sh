#!/usr/bin/env bash

set -eu -o pipefail

script_dir="$(dirname -- "${BASH_SOURCE[0]}")"

cd "$script_dir/.."
script_dir="$(pwd)" # replacing it with full path for later

set -x

cargo build --release

rm -rf target/mpvserve_dist
mkdir target/mpvserve_dist

cp -r public target/mpvserve_dist/
cp -r templates target/mpvserve_dist/
cp Rocket.toml target/mpvserve_dist/

cp target/release/mpvserve target/mpvserve_dist/

cd target
tar -cf mpvserve.tar.gz mpvserve_dist
cd "$script_dir"

mv target/mpvserve.tar.gz .
