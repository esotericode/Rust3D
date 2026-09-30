#!/bin/sh
set -eu
stride_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$stride_directory"
exec "$stride_directory/Stride" "$@"
