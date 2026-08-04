#!/bin/sh
set -eu

repository_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
build_dir="$repository_dir/.makepkg"
mkdir -p "$build_dir"

cd "$repository_dir"
BUILDDIR="$build_dir" exec /usr/bin/makepkg "$@"
