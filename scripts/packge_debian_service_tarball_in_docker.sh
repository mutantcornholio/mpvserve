#!/usr/bin/env bash

set -eu -o pipefail

script_dir="$(dirname -- "${BASH_SOURCE[0]}")"
cd "${script_dir}/debian-stretch-build"

export REPO_DIR="$(readlink -f ../..)"

image_tag="$(date +%s)"

docker build . -t "$image_tag"

docker run \
  -v "${REPO_DIR}":/mpvserve \
  -v "$(pwd)":/result \
  "$image_tag" \
  /bin/bash -c "cd /mpvserve && rm -rf target && ./scripts/package_service_tarball.sh && cp mpvserve.tar.gz /result"

cleanup() {
  exit_code=$?
  docker rmi "$image_tag"
  exit $exit_code
}
trap cleanup EXIT
