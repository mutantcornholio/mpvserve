#!/usr/bin/env bash

set -e -o pipefail

script_dir="$(dirname -- "${BASH_SOURCE[0]}")"

mkdir -p "$HOME/.bin"
cp "${script_dir}/../wrapper/mpv_wrapper.sh" "$HOME/.bin/mpv_wrapper.sh"
chmod +x "$HOME/.bin/mpv_wrapper.sh"

echo "Wrapper copied to $HOME/.bin/mpv_wrapper.sh"

XDG_APP_DIR=

if [ -n "${XDG_DATA_HOME}" ]; then
  XDG_APP_DIR="${XDG_DATA_HOME}/applications"
else
  XDG_APP_DIR="$HOME/.local/share/applications"
fi

mkdir -p "$XDG_APP_DIR"
cp "${script_dir}/../wrapper/MpvWrapper.desktop" "$XDG_APP_DIR/"

echo "MpvWrapper.desktop copied to $XDG_APP_DIR/"

ESCAPED_REPLACE=$(printf '%s\n' "$HOME" | sed -e 's/[\/&]/\\&/g')
sed -i "s/HOME_ENVIRONMENT_VARIABLE_TOKEN/$ESCAPED_REPLACE/g" "$XDG_APP_DIR/MpvWrapper.desktop"

echo "HOME in MpvWrapper.desktop substituted"

xdg-mime default MpvWrapper.desktop x-scheme-handler/mpv
update-desktop-database -v "$XDG_APP_DIR"

echo "mpv:// scheme handler installed"
