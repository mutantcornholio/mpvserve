#!/usr/bin/env bash

set -eu -o pipefail

script_dir="$(dirname -- "${BASH_SOURCE[0]}")"

cd "$script_dir/.."

platypus \
  --name MpvWrapper \
  --interface-type None \
  --app-icon "./wrapper/mpv_icon.icns" \
  --author "Cornholio" \
  --quit-after-execution \
  --uri-schemes mpv \
  "./wrapper/mpv_wrapper.sh" \
  ./MpvWrapper
