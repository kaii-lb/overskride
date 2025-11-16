#!/bin/bash

BUILD_DIR="build/"
if [ -z "$1" ]; then
    echo "Error: Target binary path not provided."
    exit 1
fi

cd "$BUILD_DIR" || { echo "Error: Cannot change to Meson build directory '$BUILD_DIR'. Please check the BUILD_DIR variable in the script."; exit 1; }

shift

echo "Running ${1} inside Meson devenv environment from directory: $(pwd)"

meson devenv -C . "sh" "-c" "$@"
