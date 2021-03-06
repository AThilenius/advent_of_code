#!/bin/bash
set -e

BASEDIR="$( cd "$(dirname "$0")" ; pwd -P )"

mkdir -p "${BASEDIR}/build"
cd "${BASEDIR}/build"
cmake ..
make
./perflab vline "${BASEDIR}/blocks-small.bmp"
