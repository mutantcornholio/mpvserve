#!/usr/bin/env bash

PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
mpv "${1/mpv\:\/\//http://}"
