#!/usr/bin/env bash

set -eu -o pipefail

script_dir="$(dirname -- "${BASH_SOURCE[0]}")"
cd "${script_dir}/debian-stretch-build"

export REPO_DIR="$(readlink -f ../..)"

image_tag="$(date +%s)"

if [ -d "$HOME/Library/Caches" ]; then
  cargo_home="$HOME/Library/Caches/cargo-docker"
elif [ -d "$XDG_CACHE_HOME" ]; then
  cargo_home="$XDG_CACHE_HOME/cargo-docker"
else
  cargo_home="$HOME/.cache/cargo-docker"
fi

mkdir -p "$cargo_home"

docker build . -t "$image_tag"

docker run \
  -v "${REPO_DIR}":/mpvserve \
  -v "$(pwd)":/result \
  -v "$cargo_home":/cache/cargo \
  "$image_tag" \
  /bin/bash -c "cd /mpvserve && ./scripts/package_service_tarball.sh && cp mpvserve.tar.gz /result"

cleanup() {
  exit_code=$?
  docker rmi --force "$image_tag"
  exit $exit_code
}
trap cleanup EXIT
